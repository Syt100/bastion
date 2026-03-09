# Change: Stream archive_v1 parts to targets to reduce local disk usage

## Why
Large backups using `archive_v1` currently stage all generated part files on local disk before uploading/copying them to the target. This makes peak local disk usage scale with the artifact size, which is impractical for multi-hundred-GB to TB workloads.

## What Changes
- Implement rolling part storage for `archive_v1`:
  - Upload/copy each finished `payload.part*` to the target as soon as it is finalized.
  - Remove the local staging part file after it has been successfully stored.
  - Preserve existing completion semantics: `manifest.json` and `complete.json` remain last.
- Apply the same behavior for both Hub-run and Agent-run filesystem backups.

## Impact
- Affected specs: backup-format, targets-webdav, targets-local-dir
- Affected code:
  - `bastion-backup` archive builder (part production + hooks)
  - `bastion-targets` target writers (WebDAV/local-dir)
  - Hub scheduler worker + Agent task runner glue

## Non-Goals
- Streaming `raw_tree_v1` uploads (still requires a local `staging/data/` tree).
- Streaming the entries index itself (still written locally as `entries.jsonl.zst`).
