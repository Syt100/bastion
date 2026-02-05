## ADDED Requirements

### Requirement: Filesystem Source Supports Snapshot Mode
Filesystem sources SHALL support a snapshot mode configuration:
- `off`: do not attempt a snapshot
- `auto`: attempt a snapshot, otherwise fall back to best-effort warnings
- `required`: fail when snapshot cannot be created

#### Scenario: Auto mode falls back with warning
- **WHEN** snapshot mode is `auto`
- **AND** no snapshot provider is available
- **THEN** the run continues using best-effort detection
- **AND** emits a `snapshot_unavailable` warning event

#### Scenario: Required mode fails when unavailable
- **WHEN** snapshot mode is `required`
- **AND** no snapshot provider is available
- **THEN** the run fails with `error_code="snapshot_unavailable"`

