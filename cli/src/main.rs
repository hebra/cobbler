use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Write};
use flume::RecvTimeoutError;
use std::time::{Duration, Instant};
use tabwriter::TabWriter;

const SERVICE_TYPE: &str = "_cobbler._tcp";
const SERVICE_DOMAIN: &str = "local.";

fn get_default_timeout() -> Duration {
    std::env::var("COBBLER_TIMEOUT")
        .ok()
        .and_then(|v| humantime::parse_duration(&v).ok())
        .unwrap_or(Duration::from_secs(60))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "help" => run_help(&args[2..]),
        "discover" => {
            if let Err(err) = run_discover(&args[2..]) {
                eprintln!("discover: {err}");
                std::process::exit(1);
            }
        }
        "status" => {
            if let Err(err) = run_status(&args[2..]) {
                eprintln!("status: {err}");
                std::process::exit(1);
            }
        }
        "packages" => {
            if let Err(err) = run_packages(&args[2..]) {
                eprintln!("packages: {err}");
                std::process::exit(1);
            }
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!();
            print_help();
            std::process::exit(1);
        }
    }
}

fn run_help(args: &[String]) {
    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "discover" => {
            let mut out = io::stdout();
            print_discover_help(&mut out);
        }
        "status" => {
            let mut out = io::stdout();
            print_status_help(&mut out);
        }
        "packages" => {
            let mut out = io::stdout();
            print_packages_help(&mut out);
        }
        "help" => print_help(),
        other => {
            eprintln!("unknown command: {other}");
            eprintln!();
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Usage: cobbler <command> [options]");
    println!();
    println!("Commands:");
    println!("  help [command]  Show help for a command");
    println!("  discover        Discover cobbler daemons on the local network");
    println!("  status          Show status of cobbler daemons");
    println!("  packages        Manage packages on cobbler daemons");
    println!();
    println!("Run `cobbler help <command>` for details.");
}

fn run_discover(args: &[String]) -> Result<(), Box<dyn Error>> {
    let timeout = parse_discover_args(args)?;

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
    let mut results: Vec<ServiceInfo> = Vec::new();
    let mut seen = HashSet::new();

    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }

        let remaining = deadline - now;
        match receiver.recv_timeout(remaining) {
            Ok(event) => {
                match event {
                    ServiceEvent::ServiceFound(service_type, fullname) => {
                        eprintln!("Found new service: {} (type: {})", fullname, service_type);
                    }
                    ServiceEvent::ServiceResolved(info) => {
                        eprintln!("Resolved service: {}", info.get_fullname());
                        let fullname = info.get_fullname().to_string();
                        if seen.insert(fullname) {
                            results.push(info);
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

    if results.is_empty() {
        println!("No cobbler daemons found.");
        return Ok(());
    }

    results.sort_by(|a, b| entry_instance(a).cmp(&entry_instance(b)));

    let stdout = io::stdout();
    let mut writer = TabWriter::new(stdout).padding(2);
    writeln!(writer, "ID\tHOST\tADDRESS\tPORT\tINSTANCE")?;
    for entry in results {
        writeln!(
            writer,
            "{}\t{}\t{}\t{}\t{}",
            entry_id(&entry),
            entry_host(&entry),
            entry_addresses(&entry),
            entry.get_port(),
            entry_instance(&entry)
        )?;
    }
    writer.flush()?;

    Ok(())
}

fn parse_discover_args(args: &[String]) -> Result<Duration, Box<dyn Error>> {
    let mut timeout = get_default_timeout();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-timeout" | "--timeout" => {
                if idx + 1 >= args.len() {
                    return Err("missing value for -timeout".into());
                }
                timeout = humantime::parse_duration(&args[idx + 1])?;
                idx += 2;
            }
            "-h" | "--help" => {
                let mut out = io::stderr();
                print_discover_help(&mut out);
                return Err("help requested".into());
            }
            other => {
                return Err(format!("unknown flag: {other}").into());
            }
        }
    }

    Ok(timeout)
}

fn print_discover_help(out: &mut dyn Write) {
    writeln!(out, "Usage: cobbler discover [options]").ok();
    writeln!(out).ok();
    writeln!(
        out,
        "Discovers services advertised as {} in {}.",
        SERVICE_TYPE, SERVICE_DOMAIN
    )
    .ok();
    writeln!(out).ok();
    writeln!(out, "Options:").ok();
    writeln!(
        out,
        "  -timeout duration   time to wait for responses (default 60s)"
    )
    .ok();
    writeln!(out, "                      The default can be overridden with COBBLER_TIMEOUT.").ok();
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

fn run_status(args: &[String]) -> Result<(), Box<dyn Error>> {
    let mut targets = Vec::new();
    let mut discover_all = false;

    if args.is_empty() {
        let mut out = io::stderr();
        print_status_help(&mut out);
        return Ok(());
    }

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => {
                discover_all = true;
            }
            "-h" | "--help" => {
                let mut out = io::stdout();
                print_status_help(&mut out);
                return Ok(());
            }
            target if !target.starts_with('-') => {
                targets.push(target.to_string());
            }
            other => {
                return Err(format!("unknown option: {other}").into());
            }
        }
        i += 1;
    }

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

fn print_status_help(out: &mut dyn Write) {
    writeln!(out, "Usage: cobbler status [options] [host:port]").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Options:").unwrap();
    writeln!(out, "  -a, --all    Get status for all discovered cobbler daemons").unwrap();
    writeln!(out, "  -h, --help   Show this help message").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Environment variables:").unwrap();
    writeln!(out, "  COBBLER_TIMEOUT  Default timeout for network operations (default 60s)").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Examples:").unwrap();
    writeln!(out, "  cobbler status -a").unwrap();
    writeln!(out, "  cobbler status localhost:8080").unwrap();
}

fn run_packages(args: &[String]) -> Result<(), Box<dyn Error>> {
    let mut targets = Vec::new();
    let mut full_upgrade = false;

    if args.is_empty() {
        let mut out = io::stderr();
        print_packages_help(&mut out);
        return Ok(());
    }

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--full-upgrade" => {
                full_upgrade = true;
            }
            "-h" | "--help" => {
                let mut out = io::stdout();
                print_packages_help(&mut out);
                return Ok(());
            }
            target if !target.starts_with('-') => {
                targets.push(target.to_string());
            }
            other => {
                return Err(format!("unknown option: {other}").into());
            }
        }
        i += 1;
    }

    if !full_upgrade {
        return Err("missing required option: --full-upgrade".into());
    }

    if targets.is_empty() {
        return Err("missing daemon endpoint".into());
    }

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

fn print_packages_help(out: &mut dyn Write) {
    writeln!(out, "Usage: cobbler packages [options] [host:port]").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Options:").unwrap();
    writeln!(out, "  --full-upgrade    Perform a full system upgrade (apt full-upgrade -y)").unwrap();
    writeln!(out, "  -h, --help        Show this help message").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "Examples:").unwrap();
    writeln!(out, "  cobbler packages --full-upgrade sam.local:8081").unwrap();
}
