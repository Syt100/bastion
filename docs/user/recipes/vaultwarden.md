# Vaultwarden backup recipe (Docker/Compose, SQLite)

This guide describes how to back up Vaultwarden when it is deployed with Docker/Compose and uses SQLite (the default for many deployments).

## Prerequisites

- Vaultwarden stores its data in a host-mounted `data/` directory.
- Bastion runs on the same host (Hub mode) OR you run an Agent on the Vaultwarden host.
- This recipe does **not** require stopping the Vaultwarden service.

## Example docker-compose.yml (Vaultwarden)

```yaml
services:
  vaultwarden:
    image: vaultwarden/server:latest
    container_name: vaultwarden
    restart: unless-stopped
    ports:
      - "8081:80"
    volumes:
      - /opt/vaultwarden/data:/data
```

In this example, the host path is `/opt/vaultwarden/data`.

## Configure the Bastion job

In Bastion Web UI:

1. Create a new job with type **Vaultwarden**
2. Set **Vaultwarden data dir** to the host path of the mounted directory:
   - Example: `/opt/vaultwarden/data`
3. Choose a target:
   - **Local directory** (simplest): `/opt/bastion-backups`
   - **WebDAV**: configure WebDAV credentials in Settings first
4. (Optional) Enable backup encryption (age)

What Bastion backs up:

- SQLite database: `<data_dir>/db.sqlite3` (snapshotted via SQLite online backup API)
- Vaultwarden data directory contents needed for restore (e.g. attachments/keys and related files)

## Verify (recommended)

Use **Verify** in the UI to run an end-to-end integrity check:

- Downloads the snapshot
- Restores it into a temporary directory
- Verifies file hashes
- Runs `PRAGMA integrity_check` for `db.sqlite3`

## Notes

- This Vaultwarden recipe currently supports SQLite only.
- If you run Vaultwarden with MySQL/PostgreSQL, wait for Bastion’s database backup primitives to support those engines before adding a Vaultwarden recipe for them.
