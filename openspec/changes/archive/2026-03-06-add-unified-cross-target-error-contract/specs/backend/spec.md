## ADDED Requirements

### Requirement: Backend SHALL emit a canonical cross-target error envelope
Backend diagnostics emitted for run failures and maintenance failures SHALL include a canonical error envelope with stable semantic fields.

#### Scenario: Target-side failure emits canonical fields
- **GIVEN** a run or maintenance step fails in a target adapter path
- **WHEN** backend appends the failure event
- **THEN** event fields SHALL include an error envelope with `schema_version`, `code`, `kind`, `retriable`, `hint`, `message`, and `transport.protocol`
- **AND** `code` and `kind` SHALL be stable, machine-readable values

### Requirement: Protocol diagnostics SHALL be transport-specific and non-HTTP-safe
Protocol-specific diagnostics SHALL be represented without forcing HTTP-only fields onto non-HTTP transports.

#### Scenario: HTTP transport failure includes HTTP status
- **GIVEN** a WebDAV or other HTTP-based target operation fails
- **WHEN** backend builds the envelope
- **THEN** `transport.protocol` SHALL be `http`
- **AND** HTTP status metadata SHALL be stored in HTTP-specific transport fields

#### Scenario: Non-HTTP transport failure excludes HTTP status
- **GIVEN** an SFTP target operation fails
- **WHEN** backend builds the envelope
- **THEN** `transport.protocol` SHALL be `sftp`
- **AND** backend SHALL NOT require an HTTP status field
- **AND** diagnostics SHALL use transport-appropriate fields (for example provider error code)

### Requirement: Retry decisions SHALL use structured retriable semantics
Retry behavior SHALL use structured envelope semantics rather than free-text message parsing as the primary source.

#### Scenario: Rate-limited failure exposes retry metadata
- **GIVEN** a target reports throttling behavior
- **WHEN** backend emits the envelope
- **THEN** `retriable.value` SHALL be `true`
- **AND** envelope SHALL include retry reason and optional retry delay metadata

### Requirement: Backend SHALL support async-operation and partial-failure diagnostics
The error envelope SHALL support asynchronous operation state and partial failure details for providers that use async APIs or batch semantics.

#### Scenario: Async provider returns accepted-then-failed operation
- **GIVEN** a cloud-drive style target reports async operation tracking
- **WHEN** an operation fails after acceptance
- **THEN** envelope context SHALL include operation metadata (for example operation id and poll hint)

#### Scenario: Batch operation partially fails
- **GIVEN** a delete or upload operation partially succeeds
- **WHEN** backend emits failure diagnostics
- **THEN** envelope context SHALL support a list of per-resource partial failures

### Requirement: Migration SHALL preserve legacy compatibility during rollout
Backend SHALL preserve existing legacy failure fields until clients are migrated.

#### Scenario: Legacy clients consume old fields during transition
- **GIVEN** a client version that still reads legacy event fields
- **WHEN** backend emits the new envelope
- **THEN** backend SHALL continue providing legacy fields during the migration window
- **AND** canonical envelope SHALL be emitted in parallel
