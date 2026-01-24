# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Non-Obvious Project Patterns

- Daemon requires Linux systems (apt-pkg-native dependency fails on other platforms)
- Uses mDNS service discovery with "_cobbler._tcp.local." service type for automatic daemon discovery
- Environment variables control daemon configuration: COBBLER_DAEMON_PORT (default 8080), COBBLER_DAEMON_HOSTNAME, COBBLER_DAEMON_IP
- Daemon runs 'apt-get update' on every status check (not cached)
- CLI uses blocking HTTP client while daemon uses async Axum framework
- Different Rust editions: CLI uses 2021, daemon uses 2024
- Container builds use podman with ports 8080 (HTTP) and 5353 (mDNS)

# Project Debug Rules (Non-Obvious Only)

- mDNS service registration failures logged with detailed error messages in daemon
- Environment variable validation with fallback warnings (env_u16 function)
- Linux-specific apt functionality debugging requires Debian-based system
- Container networking requires ports 8080 (HTTP) and 5353 (mDNS) to be exposed
- Daemon status endpoint provides JSON response with update details

# Project Documentation Rules (Non-Obvious Only)

- CLI discovers daemons via mDNS, daemon serves status via HTTP API
- Daemon only runs on Linux (Debian-based systems with apt)
- Environment variables configure daemon networking and identity
- REST and web components are planned but not yet implemented
- Container builds require both HTTP (8080) and mDNS (5353) ports

# Project Architecture Rules (Non-Obvious Only)

- Multi-component system: CLI discovers via mDNS, daemon serves HTTP status API
- Daemon architecture requires Linux (Debian-based) for apt package management
- Environment-based configuration replaces traditional config files
- mDNS service discovery enables automatic cluster discovery
- Container architecture requires both HTTP and mDNS networking

# Project Coding Rules (Non-Obvious Only)

- Use mDNS service registration patterns from daemon/src/main.rs for service discovery
- Environment variable parsing with fallbacks (env_u16 function pattern)
- Linux-specific conditional compilation with #[cfg(target_os = "linux")]
- CLI uses blocking HTTP client (reqwest with blocking feature) while daemon uses async
- Service discovery timeout handling with flume channels
- TabWriter for formatted CLI output with custom padding