# Change: Refactor filesystem tar walk module structure

## Why
`crates/bastion-backup/src/backup/filesystem/tar/walk.rs` is a large module mixing concerns (source path selection/deduping, directory walking, include/exclude evaluation, and legacy-root handling). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `tar::walk` into a folder module under `crates/bastion-backup/src/backup/filesystem/tar/walk/`
- Split implementation into focused submodules (e.g. `source_entry`, `legacy_root`) while keeping `write_tar_entries` as the stable entrypoint
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/backup/filesystem/tar/walk.rs`, `crates/bastion-backup/src/backup/filesystem/tar/walk/*.rs`

## Compatibility / Non-Goals
- No changes intended to include/exclude semantics, symlink policies, hardlink behavior, or error policy handling.

