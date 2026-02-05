# Design: SourceConsistencyReport v2

## Goals
- Reduce false negatives by using higher-resolution timestamps.
- Improve “replaced vs changed” explanations by recording both handle/path post-read fingerprints.
- Provide Windows file identity support.

## Report v2 schema

### Fingerprint v2
```
fingerprint: {
  size_bytes: u64,
  mtime_unix_nanos?: u64,        // if available
  file_id?: { ...platform... }, // best-effort
}
```

File identity:
- Unix: `{ dev: u64, ino: u64 }`
- Windows: `{ volume_serial: u64, file_index: u64 }` (best-effort)
- Other: `null`

### Sample v2
```
sample: [{
  path: string,         // archive path
  reason: string,       // e.g. "replaced", "changed", "deleted", "read_error"
  before?: fingerprint,
  after_handle?: fingerprint,
  after_path?: fingerprint,
  error?: string
}]
```

### Totals
Keep the same totals as v1 (for UI counters and stable semantics):
- `changed_total`
- `replaced_total`
- `deleted_total`
- `read_error_total`

## Detection algorithm

For each packaged file:
1) Capture `before` fingerprint from the open handle metadata (preferred) or policy-aware path metadata.
2) Stream bytes once to output while hashing (already implemented).
3) Capture `after_handle` from the same open handle metadata.
4) Capture `after_path` from policy-aware path metadata:
   - NotFound => `deleted_total += 1`
5) Compare:
   - If `before.file_id` and `after_path.file_id` exist and differ => `replaced_total += 1`
   - Else if size/mtime differ (prefer nanos when both present) => `changed_total += 1`
   - Else no warning

Reason strings should be coarse and stable for UI:
- `replaced`
- `changed`
- `deleted`
- `read_error`

Optional debug subreason (future): store a `detail` field like `mtime_changed`, `size_changed`, `file_id_changed`.

## UI rendering
- Total and breakdown based on the totals.
- Sample table should show:
  - path
  - reason
  - before vs after_path (and optionally after_handle in a details drawer)

## Testing
- Unix regression:
  - replace-via-rename after open => `replaced_total == 1`
  - verify entries index hash matches archived bytes (already exists)
- Windows regression (best-effort):
  - replace-via-rename should set `replaced_total == 1` when file_id support is present.
- Timestamp resolution:
  - add a unit test for fingerprint extraction that validates nanos are populated when available (do not rely on FS updating times).

