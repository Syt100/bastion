# Design: Backup MVP (High-Level)

## Process Model
- One executable provides:
  - Hub HTTP API + Web UI
  - Local Agent (Hub is also an Agent)
  - Optional remote Agents connected via WebSocket

## Data Directory
- Default: `<exe_dir>/data`
- Override: `BASTION_DATA_DIR`
- Contents (MVP):
  - `bastion.db` (SQLite)
  - `master.key` (master keyring)
  - `cache/` (temporary split parts, restore drill temp, etc.)

## Security Model
- Public access is supported via reverse proxy TLS termination.
- The service itself runs HTTP; Hub/Agent over LAN may run without TLS only with explicit `--insecure-http`.
- Single-user auth:
  - Password hashed with Argon2id.
  - Cookie session stored in SQLite.
  - CSRF protection enabled.
- Secrets storage:
  - `master.key` encrypts secrets stored in SQLite using AEAD.
  - Supports password-encrypted keypack export/import.
  - Supports key rotation with rollback safety (keep previous key until re-encrypt completes).

## Artifact Format (v1)
- Pipeline: tar(PAX) → zstd(level=3, threads=auto) → optional age → split into parts.
- Upload order: parts → `manifest.json` → `complete.json`.
- Without `complete.json`, a run is considered incomplete and excluded from restore points.
- `entries.jsonl.zst` stores file index (including per-file content hash for regular files).

## Hub/Agent Protocol (v1)
- Transport: WebSocket.
- Encoding: JSON.
- Must include:
  - protocol version `v`
  - message `type`
  - `msg_id` and monotonically increasing `seq` for `event` messages
- Enrollment:
  - short-lived enrollment token
  - exchange for long-lived `agent_key`
