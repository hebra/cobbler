# Cobbler daemon

Background process that runs on each node in the cluster to allow system-specific tasks.

## Development

Run the daemon locally:

```bash
go run ./cmd/cobblerd
```

Environment overrides:
- `COBBLER_DAEMON_PORT` (default `8080`)

## Discovery

The daemon advertises itself via mDNS/zeroconf as a TCP service:
- Service: `_cobbler._tcp`
- Domain: `local.`
- Instance: `cobblerd-<hostname>`
- TXT records: `id=<hostname>`
