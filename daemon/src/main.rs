use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde::Serialize;
use std::net::SocketAddr;
use std::process::Command;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_HTTP_PORT: u16 = 8080;

#[derive(Serialize, serde::Deserialize)]
struct StatusResponse {
    message: String,
    updates: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cobblerd=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let http_port = env_u16("COBBLER_DAEMON_PORT", DEFAULT_HTTP_PORT);
    let hostname = hostname_or_unknown();

    let mdns_daemon = register_mdns(http_port, &hostname);

    let app = Router::new().route("/status", get(status_handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], http_port));
    let listener = TcpListener::bind(addr).await?;

    info!(
        "cobbler daemon listening on {}",
        listener.local_addr()?
    );

    let server_result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;

    if let Err(err) = server_result {
        error!("http server error: {err}");
    }

    if let Some(mdns) = mdns_daemon {
        if let Err(err) = mdns.shutdown() {
            error!("mDNS shutdown error: {err}");
        }
    }

    Ok(())
}

async fn status_handler() -> impl IntoResponse {
    if !is_apt_available() {
        return (
            StatusCode::PRECONDITION_FAILED,
            Json(StatusResponse {
                message: "the system is not a Debian-based Linux system".to_string(),
                updates: Vec::new(),
            }),
        );
    }

    match get_apt_updates() {
        Ok(updates) => {
            let count = updates.len();
            let message = if count == 0 {
                "System is up to date".to_string()
            } else {
                format!("System has {} outdated packages", count)
            };
            (
                StatusCode::OK,
                Json(StatusResponse {
                    message,
                    updates,
                }),
            )
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(StatusResponse {
                message: format!("Failed to check for updates: {}", err),
                updates: Vec::new(),
            }),
        ),
    }
}

fn is_apt_available() -> bool {
    Command::new("apt")
        .arg("--version")
        .output()
        .is_ok()
        || Command::new("apt-get")
            .arg("--version")
            .output()
            .is_ok()
}

#[cfg(target_os = "linux")]
fn get_apt_updates() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use apt_pkg_native::Cache;

    info!("updating apt cache...");
    // To truly update we need to call 'apt-get update'.
    let _ = Command::new("apt-get")
        .arg("update")
        .output();

    info!("determining available updates...");
    let mut updates = Vec::new();
    let mut cache = Cache::get_singleton();

    let mut packages = cache.iter();
    while let Some(pkg) = packages.next() {
        let release = pkg.current_version();
        let candidate = pkg.candidate_version();

        if let (Some(rel), Some(can)) = (release, candidate) {
            if rel != can {
                updates.push(pkg.name());
            }
        }
    }

    info!("found {} available updates", updates.len());
    Ok(updates)
}

#[cfg(not(target_os = "linux"))]
fn get_apt_updates() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(vec![])
}

fn env_u16(key: &str, fallback: u16) -> u16 {
    let value = std::env::var(key).ok();
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return fallback;
    };

    match value.parse::<u16>() {
        Ok(parsed) => parsed,
        Err(_) => {
            warn!("invalid {key}={value:?}, using {fallback}");
            fallback
        }
    }
}

fn hostname_or_unknown() -> String {
    std::env::var("COBBLER_DAEMON_HOSTNAME")
        .unwrap_or_else(|_| {
            gethostname::gethostname().to_string_lossy().into_owned()
        })
        .trim_end_matches('.')
        .to_string()
}

fn register_mdns(port: u16, hostname: &str) -> Option<ServiceDaemon> {
    let daemon = match ServiceDaemon::new() {
        Ok(daemon) => {
            info!("mDNS daemon started");
            daemon
        }
        Err(err) => {
            error!("FAILED to start mDNS daemon: {err}");
            return None;
        }
    };

    let instance_hostname = hostname.split('.').next().unwrap_or(hostname);
    let instance = format!("cobblerd-{instance_hostname}");
    let host_name = format!("{instance_hostname}.local.");
    let properties = [("id", hostname)];

    info!("Registering mDNS service:");
    info!("  Instance: {}", instance);
    info!("  Host: {}", host_name);
    info!("  Port: {}", port);

    let mut info = match ServiceInfo::new(
        "_cobbler._tcp.local.",
        &instance,
        &host_name,
        "",
        port,
        &properties[..],
    ) {
        Ok(info) => {
            info!("mDNS service info created");
            info
        }
        Err(err) => {
            error!("FAILED to create mDNS service info: {err}");
            return None;
        }
    };

    if let Ok(ip) = std::env::var("COBBLER_DAEMON_IP") {
        info!("Using explicit IP from COBBLER_DAEMON_IP: {}", ip);
        let ip_addr: std::net::IpAddr = ip.parse().expect("invalid COBBLER_DAEMON_IP");
        info = match ServiceInfo::new(
            "_cobbler._tcp.local.",
            &instance,
            &host_name,
            ip_addr,
            port,
            &properties[..],
        ) {
            Ok(info) => info,
            Err(err) => {
                error!("FAILED to create mDNS service info with explicit IP: {err}");
                return None;
            }
        };
    } else {
        info!("Enabling automatic address discovery");
        info = info.enable_addr_auto();
    }

    if let Err(err) = daemon.register(info) {
        error!("FAILED to register mDNS service: {err}");
        return None;
    }

    info!("mDNS service registered successfully");
    Some(daemon)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!("failed to install Ctrl-C handler: {err}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                error!("failed to install SIGTERM handler: {err}");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_status_handler_non_linux() {
        // This test will likely run on non-linux (macOS) in this environment
        // but we can't easily fake the output of `Command::new("apt")` without mocking.
        // For now, let's just ensure it compiles and runs.
        
        let app = Router::new().route("/status", get(status_handler));
        
        let response = app
            .oneshot(Request::builder().uri("/status").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();

        // On macOS/Darwin, apt won't be available
        #[cfg(target_os = "macos")]
        assert_eq!(response.status(), StatusCode::PRECONDITION_FAILED);
        
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let status: StatusResponse = serde_json::from_slice(&body).unwrap();
        
        #[cfg(target_os = "macos")]
        {
            assert_eq!(status.message, "the system is not a Debian-based Linux system");
            assert!(status.updates.is_empty());
        }
    }
}
