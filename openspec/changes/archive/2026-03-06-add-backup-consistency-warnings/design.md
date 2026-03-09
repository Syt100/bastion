# Design: Best-effort source consistency warnings

## Goals
- Detect (best-effort) when a source file changes while it is being packaged.
- Guarantee that the per-file content hash recorded in the entries index corresponds to the bytes written into the backup output (single-read hashing).
- Surface warnings in:
  - Run events (live + historical)
  - Run summary (stable machine-readable)
  - Job run list (without N+1 run-detail requests)
- Keep the design cross-platform and bounded (sample limits).

## Data Model

### File Fingerprint (best-effort)
We record a small, cross-platform fingerprint:
- `size_bytes: u64`
- `mtime_unix_seconds: Option<u64>` (if available)
- `file_id: Option<FileId>` (best-effort; platform dependent)

`FileId`:
- Unix: `(dev, ino)` via `std::os::unix::fs::MetadataExt`
- Windows: use file index / volume serial if feasible; otherwise `None`

This fingerprint is intentionally not a cryptographic identity; it is a heuristic for "did it change while we read it".

### Consistency Report (summary + events)
We persist a structured report with bounded samples:

```
consistency: {
  changed_total: number,
  replaced_total: number,
  deleted_total: number,
  read_error_total: number,
  sample: [
    {
      path: string,        // archive path (not raw OS path)
      reason: string,      // e.g. "mtime_changed", "size_changed", "file_id_changed", "read_error"
      before?: { size_bytes, mtime_unix_seconds?, file_id? },
      after?:  { size_bytes, mtime_unix_seconds?, file_id? },
      error?: string
    }, ...
  ]
}
```

Sample is capped (e.g. 50) to avoid unbounded growth. Totals count all occurrences even when samples are truncated.

## Implementation Approach

### Single-read hashing for `archive_v1`
Current risk: hashing and tar append may read the file contents twice. If the file changes between those reads, the entries index hash can differ from the archived bytes.

Refactor tar writers to:
1. Capture `fingerprint_before` from metadata (symlink policy aware).
2. Open the file once.
3. Append to tar via `Builder::append_data(header, name, reader)` where `reader` is wrapped to:
   - stream bytes to tar
   - compute `blake3` hash over the same bytes
4. Capture `fingerprint_after` (prefer the open handle if possible; otherwise stat by path).
5. Compare and record a consistency warning when the fingerprint changes.

Notes:
- The tar header size is set from the pre-read metadata (`size_before`).
- If the file shrinks and yields an EOF before `size_before`, tar append may error; this is already governed by the job's error policy. The consistency report should still record `read_error`.
- If the file grows, tar will archive only the first `size_before` bytes; the fingerprint comparison will mark it as changed.

### `raw_tree_v1` consistency warnings
Raw-tree already reads each file once while copying + hashing. Add fingerprint-before/after around the copy and compare to record warnings.

### Vaultwarden consistency warnings
Vaultwarden packaging currently also hashes and appends paths separately. Apply the same single-read hashing strategy and record consistency warnings for files under the Vaultwarden data directory (excluding SQLite WAL/SHM/journal files which are already replaced by a snapshot).

## Reporting and Surfacing

### Run Event
If `changed_total + replaced_total + deleted_total + read_error_total > 0`:
- Emit a `run_event` with:
  - `level=warn`
  - `kind=source_consistency`
  - `message="source consistency warnings"`
  - `fields={ ...counts..., sample... }`

This is emitted once per run at the end of packaging to avoid event spam.

### Run Summary
Persist the same structured object in `summary_json`:
- filesystem job: `summary.filesystem.consistency`
- vaultwarden job: `summary.vaultwarden.consistency`

### Job Runs List API
`GET /api/jobs/:id/runs` includes:
- `consistency_changed_total` (integer, default 0)

This is derived from `summary_json` so it works for Hub- and Agent-executed runs.

## UI
- Job runs list:
  - In the status column, show a warning tag when `consistency_changed_total > 0` (include count).
- Run detail:
  - Show a warning tag when `consistency.changed_total > 0` (and optionally include other totals).
  - Encourage users to inspect the `source_consistency` event for samples.

## Testing
- Unit tests in `bastion-backup`:
  - Verify that archive hashing corresponds to the bytes written into the archive output (single-read).
  - Verify that when a file is modified during packaging, `consistency.changed_total` increments and a sample is recorded.
- UI tests:
  - Verify that `consistency_changed_total` produces a visible warning tag in the job runs list.
  - Verify that run detail renders the warning from `summary_json`.

