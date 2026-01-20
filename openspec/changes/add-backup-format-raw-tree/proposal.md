# Change: Add raw-tree backup format (no tar) with best-effort metadata preservation

## Why
The current backup payload format is a tar stream (optionally encrypted) split into parts. This is efficient for transport, but some users want an optional mode that stores data “as-is” (file tree) and keeps metadata as much as possible.

Supporting a raw-tree format also improves extensibility:
- it provides a clear separation between “artifact format” and “storage target”,
- it enables alternative restore strategies beyond tar extraction,
- it prepares the system for future targets like S3 by treating runs as structured artifacts.

## What Changes
- Add a new artifact format: `raw_tree_v1` (alongside the existing archive/tar format).
- For `raw_tree_v1`:
  - Store file contents under `data/<relative_path>` in the run directory.
  - Keep existing run metadata files at the run root (`manifest.json`, `entries.jsonl.zst`, `complete.json`).
  - Extend entries index records to include best-effort filesystem metadata (mtime/mode/uid/gid/xattrs, symlink targets, hardlink grouping).
- Add restore support for `raw_tree_v1`:
  - restore uses entries index metadata to recreate paths and apply metadata when restoring to a filesystem sink,
  - when restoring to WebDAV, store metadata under `.bastion-meta/` for round-trip recovery.
- Add a job-level option to choose artifact format:
  - `archive_v1` (default; supports encryption)
  - `raw_tree_v1` (encryption is not supported; the UI disables encryption settings when selected)

## Impact
- Affected specs: `backup-format`, `sources`, `backend`, `web-ui`
- Affected code:
  - Backup builders: `crates/bastion-backup/src/backup/filesystem/*` (and similar for other job types if extended)
  - Targets: `crates/bastion-targets/*` (store raw-tree directory content)
  - Manifest + entries index: `crates/bastion-core/src/manifest.rs`, `crates/bastion-backup/src/backup/*/entries_index.rs`
  - Restore: `crates/bastion-backup/src/restore/*`
  - UI job editor: `ui/src/components/jobs/editor/*`

## Compatibility / Non-Goals
- `raw_tree_v1` does not support payload encryption/splitting; encryption remains available only for the archive format.
- This change does not add new backup targets (it reuses existing LocalDir/WebDAV targets).

