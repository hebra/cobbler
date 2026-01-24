use axum::{http::StatusCode, routing::get, Router};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::net::SocketAddr;
use tokio::net::TcpListener;

const DEFAULT_HTTP_PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_port = env_u16("COBBLER_DAEMON_PORT", DEFAULT_HTTP_PORT);
    let hostname = hostname_or_unknown();

    let mdns_daemon = register_mdns(http_port, &hostname);

    let app = Router::new().route("/status", get(status_handler));
    let addr = SocketAddr::from(([0, 0, 0, 0], http_port));
    let listener = TcpListener::bind(addr).await?;

    eprintln!(
        "cobbler daemon listening on {}",
        listener.local_addr()?
    );

    let server_result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await;

    if let Err(err) = server_result {
        eprintln!("http server error: {err}");
    }

    if let Some(mdns) = mdns_daemon {
        if let Err(err) = mdns.shutdown() {
            eprintln!("mDNS shutdown error: {err}");
        }
    }

    Ok(())
}

async fn status_handler() -> StatusCode {
    StatusCode::OK
}

fn env_u16(key: &str, fallback: u16) -> u16 {
    let value = std::env::var(key).ok();
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return fallback;
    };

    match value.parse::<u16>() {
        Ok(parsed) => parsed,
        Err(_) => {
            eprintln!("invalid {key}={value:?}, using {fallback}");
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
            eprintln!("mDNS daemon started");
            daemon
        }
        Err(err) => {
            eprintln!("FAILED to start mDNS daemon: {err}");
            return None;
        }
    };

    let instance_hostname = hostname.split('.').next().unwrap_or(hostname);
    let instance = format!("cobblerd-{instance_hostname}");
    let host_name = format!("{instance_hostname}.local.");
    let properties = [("id", hostname)];

    eprintln!("Registering mDNS service:");
    eprintln!("  Instance: {}", instance);
    eprintln!("  Host: {}", host_name);
    eprintln!("  Port: {}", port);

    let mut info = match ServiceInfo::new(
        "_cobbler._tcp.local.",
        &instance,
        &host_name,
        "",
        port,
        &properties[..],
    ) {
        Ok(info) => {
            eprintln!("mDNS service info created");
            info
        }
        Err(err) => {
            eprintln!("FAILED to create mDNS service info: {err}");
            return None;
        }
    };

    if let Ok(ip) = std::env::var("COBBLER_DAEMON_IP") {
        eprintln!("Using explicit IP from COBBLER_DAEMON_IP: {}", ip);
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
                eprintln!("FAILED to create mDNS service info with explicit IP: {err}");
                return None;
            }
        };
    } else {
        eprintln!("Enabling automatic address discovery");
        info = info.enable_addr_auto();
    }

    if let Err(err) = daemon.register(info) {
        eprintln!("FAILED to register mDNS service: {err}");
        return None;
    }

    eprintln!("mDNS service registered successfully");
    Some(daemon)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            eprintln!("failed to install Ctrl-C handler: {err}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => {
                eprintln!("failed to install SIGTERM handler: {err}");
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
