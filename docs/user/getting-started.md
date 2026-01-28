# Quickstart

This guide assumes a small deployment (one Hub, optional Agents).

## 1. Install

Current official release artifacts are:

- **Linux**:
  - `*.tar.gz` (portable, contains the `bastion` binary; `gnu` + `musl`)
  - `*.deb` (Debian/Ubuntu; `gnu` only)
  - `*.rpm` (Fedora/RHEL/openSUSE; `gnu` only)
- **Windows**:
  - `*.zip` (portable, contains `bastion.exe`)
  - `*.msi` (installer)
- **macOS**:
  - `*.tar.gz` (portable, contains the `bastion` binary; x64 + arm64)

Examples:

- Linux `.tar.gz` / macOS `.tar.gz`:
  - `tar -xzf bastion-<version>-<target>.tar.gz`
  - `./bastion`
- Debian/Ubuntu `.deb`:
  - `sudo dpkg -i bastion-<version>-x86_64-unknown-linux-gnu.deb`
  - Run with `bastion`
- Fedora/RHEL/openSUSE `.rpm`:
  - `sudo rpm -Uvh bastion-<version>-x86_64-unknown-linux-gnu.rpm`
  - Run with `bastion`
- Windows `.msi`:
  - Install the MSI
  - Run from `C:\Program Files\Bastion\bastion.exe` (the MSI does not add PATH by default)

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

Note: this MVP currently exposes only a single-user setup flow (no user management UI yet).

## 4. Next steps

- (Optional) [Enroll Agents](/user/agents) to run backups on other machines.
- Create [Jobs](/user/jobs).
- Monitor [Runs](/user/runs) and practice recovery with [Restore and verify](/user/restore-verify).
- Configure [Storage (WebDAV)](/user/storage) if you want remote targets.
- Use [Backup snapshots](/user/backup-snapshots) for pin/delete/retention.
- (Optional) Configure [Notifications](/user/operations/notifications).
