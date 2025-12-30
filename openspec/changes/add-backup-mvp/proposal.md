# Change: Add Backup MVP (Hub/Agent + Web UI)

## Why
We want a cross-platform (Linux + Windows) backup system with a modern Web UI that can run on a single machine or centrally manage multiple machines, with remote targets (WebDAV first), notifications, integrity verification via restore drills, and first-class support for SQLite and Vaultwarden (SQLite + Docker/Compose) backups.

## What Changes
- Add a backup engine that supports:
  - File backup with tar (PAX) packaging + zstd compression.
  - Optional end-to-end encryption (age).
  - Split artifacts into configurable parts for WebDAV upload.
  - A versioned manifest + atomic completion marker.
  - Full restore verification ("restore drill") for integrity checks.
- Add SQLite online backup as a first-class source (no downtime).
- Add WebDAV as the first remote target.
- Add notifications (Email + WeCom group bot).
- Add a single-user Web UI (Vue 3) for backup/restore/verify with a modern UX.
- Add centralized management: Hub can manage multiple Agents via WebSocket + JSON protocol, using enrollment tokens.
- Add encrypted-at-rest secrets storage backed by `data/master.key`, including key export/import/rotation.

## Scope (MVP)
- Full backups only (no incremental, no dedup).
- WebDAV target only (S3/SMB later).
- Hub/Agent connection supports both secure mode behind reverse proxy TLS and an explicit insecure mode for dev/LAN.
- Public Web UI access is supported via reverse proxy (TLS termination); the service itself runs HTTP.
- Single user / single permission model (RBAC later).
- Cross-platform restore ignores owner/permission mapping for now (content correctness first).

## Out of Scope (Future)
- Incremental backups and content deduplication.
- S3/SMB/SFTP targets.
- Filesystem snapshot integration (VSS, LVM/ZFS/btrfs).
- Multi-user RBAC and audit exports.
- Built-in ACME/TLS certificate management.

## Key Decisions
- Backend language: Rust.
- Web UI: Vue 3 + TypeScript + Naive UI + Tailwind + ECharts (npm), with ESLint and unit tests.
- Backend API: HTTP + cookie session stored in SQLite + CSRF protection.
- Hub/Agent: Agent-initiated WebSocket connection, JSON messages.
- Default listen address/port: `127.0.0.1:9876` (configurable to `0.0.0.0`).
- Logs: stored in SQLite, default retention 180 days (configurable).
- Secrets: `data/master.key` encrypts secrets in SQLite; supports password-encrypted keypack export/import and key rotation.
- Compression: zstd level 3 by default (matching `zstd` CLI), threads auto.

## Risks
- Insecure (non-TLS) Hub/Agent mode can leak tokens and data; MUST require explicit opt-in and show persistent warnings.
- File semantic fidelity across platforms is complex; MVP prioritizes correctness of content and repeatable restore/verify behavior.
- Large repositories need careful DB/logging limits to avoid runaway growth; add guardrails (per-run log limits, GC).

## Success Criteria
- A user can:
  - Add an Agent to Hub (token enrollment) and run backups remotely.
  - Create and run a full backup job to WebDAV with configurable split size.
  - Receive notifications on success/failure.
  - Perform restore via Web UI wizard.
  - Run a restore drill integrity verification and view results.
  - Back up Vaultwarden data (SQLite + attachments + keys) without downtime.

