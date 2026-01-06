# Change: Update Filesystem Multi-Path Source & Partial Restore

## Why
Today, filesystem backups only support a single `filesystem.source.root` directory. This blocks:
- backing up multiple directories in one job,
- backing up individual files (a file path currently produces an almost-empty archive),
- using stable include/exclude rules across multiple inputs,
- browsing backup contents to restore only a subset of files.

## What Changes
- Extend filesystem source to support selecting **multiple paths** (files and directories).
- Define a stable **archive path** mapping for each selected source so backups preserve path structure and avoid collisions.
- Apply filesystem `include` / `exclude` rules against the **archive path** (tar-internal path) instead of root-relative paths.
- Automatically **deduplicate** overlapping selections and record a warning summary (without per-file spam).
- Add APIs to:
  - browse a node’s filesystem for multi-select source picking,
  - browse a run’s archived paths for restore selection,
  - start a restore operation with an optional “selected paths” filter.
- Update Web UI:
  - Job editor: multi-source picker (manual input + browse),
  - Restore wizard: browse + select a subset (files and directories).

## Impact
- Job spec schema changes for filesystem sources (`root` → `paths`).
- Entries index `path` semantics become **archive paths** (used by browsing + partial restore).
- Hub↔Agent control plane may need an extension to support node-scoped filesystem browsing on Agent nodes.

