# Change: Refactor backup filesystem module structure

## Why
`crates/bastion-backup/src/backup/filesystem.rs` is a large module that mixes concerns (source selection/deduplication, tar writing, entry index generation, hashing, path normalization, and JSON output). Splitting it into focused submodules makes it easier to navigate, review, and safely change.

## What Changes
- Split filesystem backup implementation into focused submodules under `crates/bastion-backup/src/backup/filesystem/`
- Keep the existing public API stable (`build_filesystem_run`, `FilesystemRunBuild`, `FilesystemBuildIssues`)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/backup/filesystem.rs`, `crates/bastion-backup/src/backup/filesystem/*.rs`

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No behavior changes intended; refactor only.

