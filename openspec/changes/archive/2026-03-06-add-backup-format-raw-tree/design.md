# Design: raw_tree_v1 format and metadata

## Artifact Formats
We treat “how a run is materialized” as an artifact format:
- `archive_v1`: existing tar+zstd(+age)+split parts
- `raw_tree_v1`: a directory tree + metadata index

The manifest records which format is used so restore/verify can select the correct implementation.

## raw_tree_v1 Storage Layout (per run)
Within the run directory (LocalDir target) / run URL (WebDAV target):
- `manifest.json`
- `entries.jsonl.zst` (entries index; JSONL; zstd-compressed)
- `complete.json`
- `data/` directory containing restored content:
  - `data/<relative_path>` for regular files
  - directories are represented by their paths and metadata records

This avoids collisions between user paths and the run’s own metadata files.

## Entries Index v2 (metadata extensions)
Existing fields remain:
- `path`, `kind`, `size`, `hash_alg`, `hash`

New optional fields (best-effort):
- `mtime` (unix seconds or nanos; TBD by implementation)
- `mode` (unix permission bits where available)
- `uid`, `gid` (where available)
- `xattrs` (map string → base64)
- `symlink_target` (string)
- `hardlink_group` (string; stable id within run)

Records are written so that older readers ignoring unknown fields continue to work.

## Restore Semantics
- Restoring to a filesystem sink applies metadata best-effort:
  - create dirs, write files, create symlinks, attempt hardlinks, then apply metadata (mtime/mode/owner/xattrs) where supported.
- Restoring to WebDAV:
  - writes content under the requested prefix,
  - writes metadata sidecar under `<prefix>/.bastion-meta/restore/<op_id>/...`

## Encryption
`raw_tree_v1` does not support payload encryption or split parts. The job editor enforces this by disabling encryption settings when raw-tree is selected.

