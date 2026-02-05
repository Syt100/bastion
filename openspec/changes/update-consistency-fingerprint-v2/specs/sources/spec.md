## MODIFIED Requirements

### Requirement: Consistency Fingerprint Captures High-Resolution Evidence
When performing best-effort source consistency detection, the system SHALL capture a v2 fingerprint that includes:
- `size_bytes`
- `mtime_unix_nanos` when available
- `file_id` when available (platform best-effort)

The system SHALL record both post-read fingerprints:
- `after_handle` (from the open handle)
- `after_path` (from the path)

#### Scenario: Same-second mutation is still detectable
- **WHEN** a file changes during packaging within the same second
- **THEN** the system can still record a `changed` warning using nanosecond mtimes when available

#### Scenario: Replace-via-rename is classified as replaced
- **WHEN** a file is replaced via rename while being packaged
- **THEN** the system records a `replaced` warning (best-effort, platform dependent)

