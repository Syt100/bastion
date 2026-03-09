## Context

`archive_v1` packaging currently writes all `payload.part*` files into `<data_dir>/runs/<run_id>/staging/`, and only after packaging completes does the system upload/copy those artifacts to the configured target.

For large runs this requires local disk roughly proportional to the final artifact size.

## Goals / Non-Goals

Goals:
- Reduce peak local disk usage for `archive_v1` backups by not retaining completed part files on disk.
- Preserve existing target semantics:
  - resumable uploads by existing file size
  - `manifest.json` and `complete.json` uploaded/written last
  - incomplete-run cleanup continues to work (no `complete.json`)

Non-Goals:
- Change the backup data format.
- Add new UI controls.

## Decisions

### Decision: Upload/copy each completed part immediately, then delete the local part

- When the archive builder finalizes a part, it emits a "part ready" event containing:
  - part file path
  - part name
  - size
- The storage layer (target) consumes these events sequentially:
  - if the destination already has a matching-size part, skip writing
  - otherwise upload/copy
  - after success, delete the local part file

This preserves the existing "resume by size" mechanism while keeping local disk bounded.

### Decision: Bound the number of in-flight part files (backpressure)

Rolling upload must not allow unbounded buffering when the target is slower than packaging.

- Use a bounded queue between packaging and target storage.
- When the queue is full, packaging blocks before producing the next part.

This bounds local disk to roughly:
- (open part being written) + (queued parts) + entries index + small metadata files.

### Decision: Keep entries index and manifest generation local; upload them at the end

- `entries.jsonl.zst` and `manifest.json` are still written in the staging directory.
- After all parts are stored and the entries index is finalized, upload/copy `entries.jsonl.zst`, then `manifest.json`, then `complete.json`.

This keeps atomic completion behavior unchanged.

## Risks / Trade-offs

- Backpressure couples packaging throughput to target throughput. This is acceptable because it is the only way to keep local disk bounded.
- Large `entries.jsonl.zst` files can still be significant for extremely large file counts; this change focuses on part files which dominate size in common cases.

## Migration Plan

- No data migration.
- Behavior is internal; existing runs remain readable.

## Open Questions

- Whether to expose a tuning knob for the queue depth / buffer size (default can be conservative).
