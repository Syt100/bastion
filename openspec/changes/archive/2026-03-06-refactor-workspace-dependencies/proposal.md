# Change: Centralize shared dependency versions via workspace dependencies

## Why
Multiple crates in this workspace repeat the same dependency versions (Tokio/Axum/SQLx/Serde/etc.). This duplication increases maintenance overhead and risks accidental version drift across crates.

## What Changes
- Introduce `[workspace.dependencies]` in the root `Cargo.toml` for shared dependencies used by multiple crates.
- Update member crates to use `workspace = true` for those dependencies, while still allowing crate-specific feature flags where needed.
- Keep behavior unchanged (this is a manifest/maintenance refactor).

## Impact
- Affected specs: `dev-workflow`
- Affected code: `Cargo.toml`, `crates/*/Cargo.toml`

