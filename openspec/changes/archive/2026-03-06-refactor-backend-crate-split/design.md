# Design: Backend crate split

## Goals
- Improve incremental debug build times by isolating high-churn code from heavy/low-churn code.
- Make future extensions (new targets, new application backup recipes) easier by defining stable crate boundaries.
- Preserve the current deployment model: a single `bastion` binary.

## Crate layout

### `bastion` (binary)
- CLI entrypoint and wiring.
- Starts hub services (HTTP server + engine background tasks) or agent mode (`bastion agent ...`).

### `bastion-core`
- Shared types that are reused across multiple crates and are intentionally kept lightweight.
- Examples: job spec types, agent protocol types, small error wrappers, manifest structures.

### `bastion-config`
- Runtime configuration types shared by hub components (bind, data dir, trusted proxies, retention settings).
- Data directory resolution helpers.
- Does NOT own CLI parsing; CLI stays in the `bastion` binary crate.

### `bastion-storage`
- SQLite/sqlx initialization and repository modules (`*_repo.rs`), including auth/session storage and encrypted secrets storage.
- Secrets crypto/keyring/keypack logic.

### `bastion-targets`
- Backup targets implementations (local directory, WebDAV). Future targets (S3/SMB) land here.
- Upload/download primitives, multipart support, and target-specific metadata handling.

### `bastion-backup`
- Backup sources and payload building (filesystem/sqlite/vaultwarden), tar + zstd, optional payload encryption helpers.
- Restore primitives and verification helpers (as used by restore wizard / restore drill).

### `bastion-notify`
- Notification delivery (Email, WeCom). Future notifiers land here.

### `bastion-engine`
- Scheduler/run lifecycle orchestration, maintenance loops, run events bus.
- Agent manager + agent task dispatch (hub side), and agent client runtime (agent mode) if needed.

### `bastion-http`
- Axum router and HTTP/WS handlers, auth middleware glue.
- Optional embedded UI assets feature (`embed-ui`).

## Dependency rules (layering)
- The dependency graph MUST be acyclic.
- `bastion-core` MUST NOT depend on any of: `axum`, `tokio`, `reqwest`, `sqlx`.
- `bastion-http` depends on `bastion-engine` (and `bastion-config`) rather than reaching into storage/backup/targets directly.
- `bastion-engine` depends on `bastion-storage`, `bastion-backup`, `bastion-targets`, and `bastion-notify`.
- `bastion-*` crates MAY depend on `bastion-core` for shared types.

## Migration approach
1. Move shared types (job spec, agent protocol, etc.) to `bastion-core`.
2. Introduce new crates and move modules in small chunks, keeping compilation green after each chunk.
3. Keep the `bastion` binary minimal and treat it as wiring.

