use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Write};
use flume::RecvTimeoutError;
use std::time::{Duration, Instant};
use tabwriter::TabWriter;

const SERVICE_TYPE: &str = "_cobbler._tcp";
const SERVICE_DOMAIN: &str = "local.";

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
                if let ServiceEvent::ServiceResolved(info) = event {
                    let fullname = info.get_fullname().to_string();
                    if seen.insert(fullname) {
                        results.push(info);
                    }
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
    let mut timeout = Duration::from_secs(3);
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
        "  -timeout duration   time to wait for responses (default 3s)"
    )
    .ok();
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
