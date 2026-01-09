# Change: Refactor filesystem tar module entrypoint structure

## Why
`crates/bastion-backup/src/backup/filesystem/tar.rs` currently serves as the module entrypoint while its submodules already live under `crates/bastion-backup/src/backup/filesystem/tar/`. Converting the entrypoint into `tar/mod.rs` aligns the layout with the existing directory structure and makes navigation more consistent.

## What Changes
- Move `backup/filesystem/tar.rs` to `backup/filesystem/tar/mod.rs`
- Keep existing tar submodules (`entry`, `walk`) under `backup/filesystem/tar/`
- Preserve behavior and internal module API (`backup::filesystem::tar::*`)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/backup/filesystem/tar.rs`, `crates/bastion-backup/src/backup/filesystem/tar/mod.rs`

## Compatibility / Non-Goals
- No behavior changes intended for tar writing, compression, or encryption paths.

