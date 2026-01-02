# Change: Refactor backend into focused crates (incremental debug builds + extensibility)

## Why
The backend is currently implemented as a large single crate (`crates/bastion`). Even small edits trigger a relatively slow rebuild, which hurts iteration speed (especially in debug builds). In addition, as we add more targets (S3/SMB) and more application recipes, keeping all logic in one crate increases coupling and makes future changes riskier.

## What Changes
- Split the backend implementation into focused Rust crates with clear responsibilities (HTTP, engine/scheduler, storage, backup, targets, notifications, shared types).
- Keep the product experience the same: a single `bastion` binary that can run hub mode or agent mode.
- Keep behavior/API compatible: no route changes, no DB schema changes beyond what is required to move code.

## Impact
- Affected specs: `backend`, `development-workflow`
- Affected code:
  - Workspace `Cargo.toml` and backend crate manifests
  - New crates under `crates/`
  - `crates/bastion/src/*` (moved into new crates)

## Compatibility / Non-Goals
- No changes to the Web UI, HTTP routes, JSON schemas, or backup formats.
- No changes to authentication/authorization semantics.
- No effort to optimize link time in this change (handled separately if needed).

