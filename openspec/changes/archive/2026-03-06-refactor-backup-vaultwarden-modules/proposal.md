# Change: Refactor vaultwarden backup module structure

## Why
`crates/bastion-backup/src/backup/vaultwarden.rs` is a large module mixing concerns (run orchestration, snapshot creation, tar/part writing, directory walking + entry indexing, hashing, json IO, and tests). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `backup::vaultwarden` into a folder module under `crates/bastion-backup/src/backup/vaultwarden/`
- Split implementation into focused submodules (e.g. `builder`, `tar`, `io`, `hash`)
- Keep the existing public surface stable (`backup::vaultwarden::build_vaultwarden_run`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/backup/vaultwarden.rs`, `crates/bastion-backup/src/backup/vaultwarden/*.rs`

## Compatibility / Non-Goals
- No changes intended to archive contents, entry index format, encryption behavior, or snapshot semantics.

