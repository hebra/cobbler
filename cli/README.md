# Cobbler CLI

The Cobbler CLI (`cobbler`) is a command-line interface for managing Cobbler daemons across your network. It uses mDNS for automatic discovery and interacts with daemons via their REST API.

## Installation

To build the CLI:

```bash
cargo build --release
```

The binary will be located at `target/release/cobbler`.

## Usage

### Discovery

Discover all Cobbler daemons on the local network:

```bash
cobbler discover [--timeout <seconds>]
```

### Status

Check the status of one or more daemons:

```bash
# Check all discovered daemons
cobbler status --all

# Check specific daemons
cobbler status <host:port> [<host:port> ...]
```

### Package Management

Trigger a full system upgrade on target nodes:

```bash
cobbler packages --full-upgrade <target> [<target> ...]
```

## Configuration

The CLI can be configured via environment variables:

- `COBBLER_TIMEOUT`: Default timeout for network operations (e.g., `30s`, `1m`). Default is `60s`.

## Development

### Running Tests

```bash
cargo test
```
