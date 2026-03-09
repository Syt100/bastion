# Change: Refactor filesystem tar writer module structure

## Why
`crates/bastion-backup/src/backup/filesystem/tar.rs` is a large module that mixes concerns (part/encryption orchestration, source walking, entry writing, and hardlink handling). Splitting it into focused submodules improves readability and maintainability.

## What Changes
- Split filesystem tar implementation into focused submodules under `crates/bastion-backup/src/backup/filesystem/tar/`
- Keep existing public surface stable (`write_tar_zstd_parts` and behavior)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/backup/filesystem/tar.rs`, `crates/bastion-backup/src/backup/filesystem/tar/*.rs`

## Compatibility / Non-Goals
- No changes intended to backup output format or entry index format.
- No changes intended to include/exclude semantics, symlink policies, or hardlink behavior.

