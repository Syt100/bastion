# Quickstart

This guide assumes a small deployment (one Hub, optional Agents).

## 1. Install

Current official release artifacts are:

- **Linux**: `*.tar.gz` (contains the `bastion` binary)
- **Windows**: `*.zip` (contains `bastion.exe`)

You can also build from source (see [Developer docs](/dev/)).

## 2. Run the Hub

Run the Hub on the machine that will orchestrate backups:

```bash
./bastion
```

Defaults:

- Binds to `127.0.0.1:9876`
- Stores state in a **data directory** (SQLite + encrypted secrets)

Useful options:

- `--host <ip>` / `BASTION_HOST=<ip>`
- `--port <port>` / `BASTION_PORT=<port>`
- `--data-dir <path>` / `BASTION_DATA_DIR=<path>`

> For LAN/dev (no TLS), you can bind to a non-loopback address with `--insecure-http` / `BASTION_INSECURE_HTTP=1`.
> For public access, put Bastion behind a reverse proxy that terminates TLS (see [Reverse proxy](/user/operations/reverse-proxy)).

## 3. First-run setup (create the first user)

Open the Web UI:

- `http://127.0.0.1:9876/`

On first launch, Bastion requires initialization (create the first user). After that, you can log in normally.

## 4. Next steps

- (Optional) [Enroll Agents](/user/agents) to run backups on other machines.
- Create [Jobs](/user/jobs).
- Configure [Storage (WebDAV)](/user/storage) if you want remote targets.
- Use [Backup snapshots](/user/backup-snapshots) for pin/delete/retention.

