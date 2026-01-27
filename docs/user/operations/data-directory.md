# Data directory layout and key management

Bastion stores its state and secrets in a configurable data directory.

## Where is the data directory?

Priority order:

1. `--data-dir <path>` (CLI)
2. `BASTION_DATA_DIR=<path>` (env)
3. `<exe_dir>/data` if writable
4. OS fallback directory:
   - Windows: `%PROGRAMDATA%\\bastion\\data` (if available)
   - Otherwise: OS-specific app local data dir (via `directories` crate)

## What files are inside?

Common files/directories:

- `bastion.db` — SQLite database (jobs, runs, secrets metadata, etc.)
- `master.key` — local master keyring (used to encrypt secrets at rest in `bastion.db`)
- `runs/` — temporary per-run staging directories while building/uploading artifacts
  - Incomplete runs may remain if the process is interrupted.

Agent mode also stores:

- `agent.json` — agent enrollment identity (agent_id/agent_key)

## Backing up the data directory

At minimum, you SHOULD back up:

- `master.key`
- `bastion.db`

If you lose `master.key`, you will not be able to decrypt existing encrypted secrets stored in the database (WebDAV credentials, SMTP, WeCom webhooks, backup encryption identities, etc.).

## Keypack export/import (recommended for backups)

Bastion provides a password-encrypted “keypack” export for `master.key`.

Export keypack:

```bash
./bastion keypack export --out /secure/location/bastion-keypack.json --password-stdin
```

Import keypack:

```bash
./bastion keypack import --in /secure/location/bastion-keypack.json --password-stdin
```

Import with overwrite (dangerous):

```bash
./bastion keypack import --in /secure/location/bastion-keypack.json --password-stdin --force
```

After importing or rotating `master.key`, restart the service to ensure the new keyring is loaded.

## Master key rotation

Rotate the active key in `master.key`:

```bash
./bastion keypack rotate
```

Rotation keeps old keys so existing secrets remain decryptable; new secrets use the new active key.

