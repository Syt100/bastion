# Change: Refactor backup filesystem module entrypoint structure

## Why
`crates/bastion-backup/src/backup/filesystem.rs` currently serves as the module entrypoint while its implementation submodules already live under `crates/bastion-backup/src/backup/filesystem/`. Converting the entrypoint into `filesystem/mod.rs` aligns the layout with the existing directory structure and makes navigation more consistent.

## What Changes
- Move `backup/filesystem.rs` to `backup/filesystem/mod.rs`
- Move filesystem unit tests into `backup/filesystem/tests.rs`
- Preserve the existing public API (`bastion_backup::backup::filesystem::*`) and behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/backup/filesystem.rs`, `crates/bastion-backup/src/backup/filesystem/mod.rs`

## Compatibility / Non-Goals
- No behavior changes intended for filesystem backup building, manifests, or entry index generation.

