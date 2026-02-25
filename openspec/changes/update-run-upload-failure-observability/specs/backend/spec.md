## ADDED Requirements

### Requirement: Rolling upload failures preserve uploader root cause
When rolling archive upload is enabled, the backend SHALL preserve uploader root-cause diagnostics when packaging observes uploader channel drop.

#### Scenario: Uploader fails before next part send
- **GIVEN** a rolling upload run and uploader task encounters a WebDAV upload error
- **WHEN** the packaging thread finalizes a subsequent part and sender `blocking_send` fails
- **THEN** the resulting run failure message includes uploader root cause details
- **AND** the message SHALL NOT degrade to only `rolling uploader dropped`

### Requirement: Packaging and uploader outcomes are reconciled deterministically
The backend SHALL always reconcile both packaging and uploader outcomes before finalizing a failed run.

#### Scenario: Packaging fails and uploader also fails
- **GIVEN** packaging path returns an error
- **AND** uploader join handle returns an error
- **WHEN** the run error is finalized
- **THEN** backend chooses a deterministic root-cause-first failure message
- **AND** preserves secondary failure details in diagnostics fields

### Requirement: Run failed events include structured diagnostics
Final run `failed` events SHALL include structured, machine-readable diagnostics fields.

#### Scenario: HTTP payload limit failure on WebDAV PUT
- **GIVEN** WebDAV PUT fails with HTTP 413 during rolling upload
- **WHEN** run terminalizes as failed
- **THEN** failed event fields include at least error code/kind, HTTP status, part metadata, and operator hint to reduce `part_size_bytes` or increase gateway limits

#### Scenario: Transport timeout or connection reset
- **GIVEN** WebDAV upload fails due to timeout/connectivity issue
- **WHEN** run terminalizes as failed
- **THEN** failed event fields include timeout/network classification and hint for timeout/retry tuning

### Requirement: Archive writer failure wrapping preserves source chain
Archive write failures SHALL preserve source error chain (instead of string-only flattening) so downstream classifiers can inspect concrete causes.

#### Scenario: Callback io::Error wraps rolling uploader diagnostic
- **GIVEN** archive write path receives callback `io::Error` containing rolling upload diagnostic
- **WHEN** archive layer maps it to `anyhow::Error`
- **THEN** classifier can still access inner cause via error chain traversal

### Requirement: WebDAV upload tuning supports timeout and retry controls
WebDAV request limits SHALL support optional timeout/retry tuning fields with validation and backward-compatible defaults.

#### Scenario: Existing job spec omits new tuning fields
- **GIVEN** a job spec without timeout/retry tuning fields
- **WHEN** job is validated and executed
- **THEN** backend uses defaults compatible with prior behavior

#### Scenario: Operator configures tighter/larger timeouts
- **GIVEN** job spec includes timeout/retry tuning values
- **WHEN** upload requests are executed
- **THEN** WebDAV client honors configured values within validated bounds
