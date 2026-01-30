# Upgrade and rollback

This guide is written for small deployments (personal / family / small teams) and focuses on avoiding data loss.

## Key idea: treat upgrades as potentially irreversible

Bastion uses SQLite migrations. A newer version may migrate the database to a newer schema.

Rollback is **not guaranteed** unless you restore a backup of the data directory taken before the upgrade.

## Before upgrading

1. Identify your **data directory** (SQLite + `master.key`):
   - See: [Data directory](/user/operations/data-directory)
2. Stop writes:
   - Stop the Hub process / service.
   - Do not run multiple Bastion versions against the same data directory.
3. Back up the data directory (recommended minimum):
   - `bastion.db`
   - `master.key`
4. (Optional, recommended) Export a password-encrypted keypack:

```bash
bastion keypack export --out /secure/location/bastion-keypack.json --password-stdin
```

## Verify after upgrading

After upgrade, verify:

- The service is running (systemd / Windows Service / container status).
- The Web UI loads.
- Agents reconnect (if you use them).
- Health endpoints:
  - `GET /api/health` (liveness)
  - `GET /api/ready` (readiness)
- Run `bastion doctor` on the host (if available) to catch common misconfigurations.

## Upgrade by installation method

### Docker / containers (if you run Bastion in a container)

Assumptions:
- Your data directory is persisted via a Docker volume or bind mount.

Steps:

1. Stop the container:
   - `docker compose down` (or `docker stop ...`)
2. Back up the volume / bind mount (copy it somewhere safe).
3. Update the image tag (pin to a specific version), then start:
   - `docker compose pull && docker compose up -d`
4. Verify (see above).

Rollback:

1. Stop the container.
2. Restore the volume/bind mount backup.
3. Start the container using the previous image tag.

### Linux packages (.deb/.rpm + systemd)

Steps:

1. Stop the service:

```bash
sudo systemctl stop bastion
```

2. Back up the data directory:
   - Default for packages: `/var/lib/bastion` (via `/etc/bastion/bastion.env`)
3. Install the new package:
   - Debian/Ubuntu: `sudo dpkg -i bastion-<version>-x86_64-unknown-linux-gnu.deb`
   - Fedora/RHEL/openSUSE: `sudo rpm -Uvh bastion-<version>-x86_64-unknown-linux-gnu.rpm`
4. Reload systemd units (safe even if unchanged):

```bash
sudo systemctl daemon-reload
```

5. Start the service (packages do not auto-start):

```bash
sudo systemctl start bastion
```

Rollback:

1. Stop the service.
2. Reinstall the previous package version.
3. Restore the data directory backup (taken before upgrade).
4. Start the service again.

### Windows MSI (service install)

Steps:

1. Stop the service:
   - Services app (`services.msc`) -> `Bastion` -> Stop
   - Or: `sc stop Bastion`
2. Back up the data directory:
   - Common default: `%PROGRAMDATA%\\bastion\\data`
3. Install the new MSI.
4. Start the service:
   - `services.msc` -> Start
   - Or: `sc start Bastion`

Rollback:

1. Stop the service.
2. Reinstall the previous MSI version (or uninstall then install the older MSI).
3. Restore the data directory backup.
4. Start the service again.

### Portable tar/zip installs

Recommended approach:
- Keep a stable data directory using `BASTION_DATA_DIR`.
- Store each release in a versioned directory (so rollback is “swap the binary”).

Steps:

1. Stop the process.
2. Back up the data directory.
3. Replace the binary with the new version.
4. Start the new version.

Rollback:

1. Stop the process.
2. Restore the data directory backup.
3. Start the previous binary version again.
