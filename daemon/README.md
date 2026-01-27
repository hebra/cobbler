# Cobbler Daemon

The Cobbler Daemon (`cobblerd`) is a background service that runs on managed Linux nodes. It provides a REST API for system status and package management, specifically targeting Debian-based systems using APT.

## Features

- **mDNS Registration**: Automatically announces itself on the local network as `_cobbler._tcp`.
- **System Status**: Reports whether the system is up-to-date and lists available updates.
- **Package Management**: Can trigger a full system upgrade via APT.
- **Port Hunting**: Automatically finds an available port starting from 8080 if not specified.

## Installation

### From Source

```bash
cargo build --release
```

The binary will be located at `target/release/cobblerd`.

### Using Docker/Podman

A `Containerfile` is provided for building a container image:

```bash
podman build -t cobblerd .
podman run -d --net=host --cap-add=CAP_SYS_ADMIN cobblerd
```

*Note: `--net=host` is required for mDNS discovery, and `CAP_SYS_ADMIN` (or equivalent) may be needed for APT operations depending on your configuration.*

## Configuration

Environment variables can be used for configuration:

- `COBBLER_DAEMON_PORT`: Port to listen on.
- `COBBLER_DAEMON_HOSTNAME`: Hostname to use for mDNS registration.
- `COBBLER_DAEMON_IP`: Explicit IP address to use for mDNS registration.
- `RUST_LOG`: Logging level (e.g., `info`, `debug`).

## API Endpoints

### `GET /status`

Returns the current system status.

**Response:**
```json
{
  "message": "System has 2 outdated packages",
  "updates": ["libc6", "vim"],
  "is_upgrading": false
}
```

### `POST /packages/full-upgrade`

Triggers a full system upgrade (`apt full-upgrade -y`). This operation is asynchronous.

**Response:**
```json
{
  "message": "full upgrade triggered"
}
```

## Development

### Running Tests

```bash
cargo test
```

*Note: Some tests are platform-specific and may behave differently on non-Linux systems.*
