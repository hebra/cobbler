use clap::{Parser, Subcommand};
use flume::RecvTimeoutError;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use tabwriter::TabWriter;

const SERVICE_TYPE: &str = "_cobbler._tcp";
const SERVICE_DOMAIN: &str = "local.";

fn get_default_timeout() -> Duration {
    std::env::var("COBBLER_TIMEOUT")
        .ok()
        .and_then(|v| {
            v.parse::<u64>()
                .map(Duration::from_secs)
                .ok()
                .or_else(|| humantime::parse_duration(&v).ok())
        })
        .unwrap_or(Duration::from_secs(60))
}

#[derive(Parser)]
#[command(name = "cobbler")]
#[command(about = "A CLI tool for cobbler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover cobbler daemons on the local network
    Discover {
        /// Time to wait for responses in seconds
        #[arg(short, long, default_value = "5", env = "COBBLER_TIMEOUT")]
        timeout: u64,
    },
    /// Show status of cobbler daemons
    Status {
        /// Get status for all discovered cobbler daemons
        #[arg(short, long)]
        all: bool,

        /// Targets (host:port)
        targets: Vec<String>,
    },
    /// Manage packages on cobbler daemons
    Packages {
        /// Perform a full system upgrade
        #[arg(long, required = true)]
        full_upgrade: bool,

        /// Targets (host:port)
        #[arg(required = true, num_args = 1..)]
        targets: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Discover { timeout } => run_discover(Duration::from_secs(timeout)),
        Commands::Status { all, targets } => run_status(all, targets),
        Commands::Packages {
            full_upgrade,
            targets,
        } => run_packages(full_upgrade, targets),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_discover(timeout: Duration) -> Result<(), Box<dyn Error>> {
    println!("Discovery will take {} seconds", timeout.as_secs());
    let mdns = ServiceDaemon::new().map_err(|err| format!("create resolver: {err}"))?;
    let service_name = format!(
        "{}.{}",
        SERVICE_TYPE.trim_end_matches('.'),
        SERVICE_DOMAIN
    );
    let receiver = mdns
        .browse(&service_name)
        .map_err(|err| format!("browse: {err}"))?;

    let deadline = Instant::now() + timeout;
    let mut seen = HashSet::new();
    let mut header_printed = false;

    let stdout = io::stdout();
    let mut writer = TabWriter::new(stdout).padding(2);

    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }

        let remaining = deadline - now;
        match receiver.recv_timeout(remaining) {
            Ok(event) => {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let fullname = info.get_fullname().to_string();
                        if seen.insert(fullname) {
                            if !header_printed {
                                writeln!(writer, "ID\tHOST\tADDRESS\tPORT\tINSTANCE")?;
                                header_printed = true;
                            }
                            writeln!(
                                writer,
                                "{}\t{}\t{}\t{}\t{}",
                                entry_id(&info),
                                entry_host(&info),
                                entry_addresses(&info),
                                info.get_port(),
                                entry_instance(&info)
                            )?;
                            writer.flush()?;
                        }
                    }
                    ServiceEvent::SearchStopped(service_type) => {
                        eprintln!("Search stopped for {}", service_type);
                    }
                    _ => {}
                }
            }
            Err(RecvTimeoutError::Timeout) => break,
            Err(RecvTimeoutError::Disconnected) => {
                return Err("browse: receiver disconnected".into());
            }
        }
    }

    let _ = mdns.shutdown();

    if !header_printed {
        println!("No cobbler daemons found.");
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parse_discover_default() {
        let cli = Cli::parse_from(&["cobbler", "discover"]);
        if let Commands::Discover { timeout } = cli.command {
            assert_eq!(timeout, 5);
        } else {
            panic!("Wrong command");
        }
    }

    #[test]
    fn test_cli_parse_discover_timeout() {
        let cli = Cli::parse_from(&["cobbler", "discover", "-t", "10"]);
        if let Commands::Discover { timeout } = cli.command {
            assert_eq!(timeout, 10);
        } else {
            panic!("Wrong command");
        }
    }

    #[test]
    fn test_get_default_timeout() {
        std::env::set_var("COBBLER_TIMEOUT", "15");
        assert_eq!(get_default_timeout(), Duration::from_secs(15));

        std::env::set_var("COBBLER_TIMEOUT", "1m");
        assert_eq!(get_default_timeout(), Duration::from_secs(60));

        std::env::remove_var("COBBLER_TIMEOUT");
        assert_eq!(get_default_timeout(), Duration::from_secs(60));
    }
}


fn entry_id(entry: &ServiceInfo) -> String {
    let props = entry.get_properties();
    props
        .get("id")
        .map(|value| value.to_string())
        .unwrap_or_default()
}

fn entry_host(entry: &ServiceInfo) -> String {
    entry.get_hostname().trim_end_matches('.').to_string()
}

fn entry_addresses(entry: &ServiceInfo) -> String {
    let mut parts = Vec::new();
    let addrs = entry.get_addresses();
    for addr in addrs.iter().filter(|addr| addr.is_ipv4()) {
        parts.push(addr.to_string());
    }
    for addr in addrs.iter().filter(|addr| addr.is_ipv6()) {
        parts.push(addr.to_string());
    }
    parts.join(",")
}

fn entry_instance(entry: &ServiceInfo) -> String {
    let fullname = entry.get_fullname();
    let suffix = format!(
        ".{}.{}",
        SERVICE_TYPE.trim_end_matches('.'),
        SERVICE_DOMAIN
    );
    fullname
        .strip_suffix(&suffix)
        .unwrap_or(fullname)
        .to_string()
}

fn run_status(discover_all: bool, mut targets: Vec<String>) -> Result<(), Box<dyn Error>> {
    if discover_all {
        targets.extend(discover_targets()?);
    }

    if targets.is_empty() {
        println!("No targets found.");
        return Ok(());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(get_default_timeout())
        .build()?;

    let mut tw = TabWriter::new(io::stdout());
    writeln!(tw, "TARGET\tSTATUS")?;

    for target in targets {
        let url = resolve_url(&target);
        let status_url = format!("{}/status", url);

        let (status, body) = match client.get(&status_url).send() {
            Ok(resp) => {
                let status = resp.status().to_string();
                let body = match resp.json::<serde_json::Value>() {
                    Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Failed to pretty-print JSON".to_string()),
                    Err(_) => "Could not parse response as JSON".to_string(),
                };
                (status, body)
            }
            Err(err) => (format!("Error: {}", err), "".to_string()),
        };

        writeln!(tw, "{}\t{}", target, status)?;
        if !body.is_empty() {
            writeln!(tw, "\t{}", body.replace('\n', "\n\t"))?;
        }
    }

    tw.flush()?;

    Ok(())
}

fn discover_targets() -> Result<Vec<String>, Box<dyn Error>> {
    let mut targets = Vec::new();
    let mdns = ServiceDaemon::new().map_err(|err| format!("create resolver: {err}"))?;
    let service_name = format!("{}.{}", SERVICE_TYPE.trim_end_matches('.'), SERVICE_DOMAIN);
    let receiver = mdns
        .browse(&service_name)
        .map_err(|err| format!("browse: {err}"))?;

    let timeout = get_default_timeout();
    let deadline = Instant::now() + timeout;
    let mut seen = HashSet::new();

    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }

        let remaining = deadline - now;
        match receiver.recv_timeout(remaining) {
            Ok(event) => {
                if let ServiceEvent::ServiceResolved(info) = event {
                    for addr in info.get_addresses() {
                        let target = format!("{}:{}", addr, info.get_port());
                        if seen.insert(target.clone()) {
                            targets.push(target);
                        }
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => break,
            Err(err) => return Err(format!("mDNS error: {err}").into()),
        }
    }
    Ok(targets)
}

fn resolve_url(target: &str) -> String {
    if target.starts_with("http://") || target.starts_with("https://") {
        target.trim_end_matches('/').to_string()
    } else if target.contains(':') && target.split(':').last().unwrap().chars().all(|c| c.is_ascii_digit()) {
        let parts: Vec<&str> = target.split(':').collect();
        let host = parts[..parts.len() - 1].join(":");
        let port = parts.last().unwrap();

        if host.contains(':') && !host.starts_with('[') {
            format!("http://[{}]:{}", host, port)
        } else {
            format!("http://{}:{}", host, port)
        }
    } else {
        format!("http://{}", target.trim_end_matches('/'))
    }
}


fn run_packages(_full_upgrade: bool, targets: Vec<String>) -> Result<(), Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(get_default_timeout())
        .build()?;

    let mut tw = TabWriter::new(io::stdout());
    writeln!(tw, "TARGET\tSTATUS")?;

    for target in targets {
        let url = resolve_url(&target);
        let upgrade_url = format!("{}/packages/full-upgrade", url);

        let (status, body) = match client.post(&upgrade_url).send() {
            Ok(resp) => {
                let status = resp.status().to_string();
                let body = match resp.json::<serde_json::Value>() {
                    Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Failed to pretty-print JSON".to_string()),
                    Err(_) => "Upgrade triggered successfully".to_string(),
                };
                (status, body)
            }
            Err(err) => (format!("Error: {}", err), "".to_string()),
        };

        writeln!(tw, "{}\t{}", target, status)?;
        if !body.is_empty() {
            writeln!(tw, "\t{}", body.replace('\n', "\n\t"))?;
        }
    }

    tw.flush()?;

    Ok(())
}

