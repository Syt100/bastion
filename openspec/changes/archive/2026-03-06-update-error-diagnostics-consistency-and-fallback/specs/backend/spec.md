## ADDED Requirements

### Requirement: WebDAV read and delete failures preserve actionable diagnostics
The backend SHALL preserve actionable transport diagnostics for WebDAV read/delete paths, including status semantics needed for classification and retry decisions.

#### Scenario: Reader receives authentication failure
- **GIVEN** WebDAV artifact read/download fails with HTTP 401 or 403
- **WHEN** the driver bridge maps the error into `DriverError`
- **THEN** the error SHALL be classified as `auth`
- **AND** the failure message SHALL include enough context to identify the HTTP status

#### Scenario: Reader receives retriable gateway failure
- **GIVEN** WebDAV read/download fails with HTTP 429 or 503
- **WHEN** retry logic evaluates the error
- **THEN** the error SHALL be treated as retriable network/upstream failure
- **AND** retry delay metadata (if present) SHALL be preserved for diagnostics

### Requirement: Driver error-kind mapping remains semantically consistent
Driver-level error mapping SHALL keep auth/config/network classes consistent with underlying WebDAV diagnostics rather than flattening all failures into `network`.

#### Scenario: Missing artifact path maps to configuration-class failure
- **GIVEN** target reader requests an invalid or missing artifact path and receives HTTP not-found semantics
- **WHEN** the bridge classifies the failure
- **THEN** it SHALL map to a non-network class (`config` or equivalent not-found configuration class)
- **AND** cleanup/run orchestration SHALL be able to block-or-handle it differently from transient network faults

### Requirement: Run failure fallback hints avoid transport-specific misdirection
When a failed run does not include explicit WebDAV PUT diagnostics, fallback hints SHALL remain actionable without assuming WebDAV transport as the root cause.

#### Scenario: Unknown non-WebDAV failure
- **GIVEN** final run failure lacks recognized transport classifiers
- **WHEN** fallback fields are generated
- **THEN** `error_kind` SHALL be `unknown`
- **AND** hint text SHALL use generic troubleshooting guidance instead of WebDAV-specific instructions

#### Scenario: Storage capacity exhaustion is detected
- **GIVEN** error-chain text indicates disk full, no space left, quota exceeded, or insufficient storage
- **WHEN** fallback fields are generated
- **THEN** `error_kind` SHALL indicate storage-capacity failure
- **AND** hint text SHALL direct operators to free capacity or adjust retention

### Requirement: Cleanup and artifact-delete events include actionable hints
Maintenance task failure events SHALL include machine-readable hints alongside error kind so operators can resolve blocked/retrying states from UI events.

#### Scenario: Cleanup task is blocked by credentials
- **GIVEN** incomplete-cleanup or artifact-delete fails with auth/config classification
- **WHEN** failure/blocked/abandoned event is appended
- **THEN** event fields SHALL include `error_kind` and `hint`
- **AND** hint SHALL describe next action (for example fixing credentials or target path configuration)
