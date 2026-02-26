## ADDED Requirements

### Requirement: Web UI SHALL render canonical error envelope diagnostics
Web UI diagnostics panels SHALL prioritize canonical envelope fields when present.

#### Scenario: Event contains canonical envelope
- **GIVEN** a run event includes canonical envelope diagnostics
- **WHEN** user opens event details
- **THEN** UI SHALL render localized message and hint from envelope keys and params
- **AND** UI SHALL fall back gracefully when localization keys are missing

### Requirement: Web UI SHALL display protocol-specific details by transport
Web UI SHALL render transport-specific detail rows using envelope transport metadata.

#### Scenario: HTTP event shows HTTP diagnostics
- **GIVEN** envelope `transport.protocol` is `http`
- **WHEN** event details are rendered
- **THEN** UI SHALL display HTTP-specific diagnostics (for example status and retry delay) when available

#### Scenario: SFTP event shows provider diagnostics without HTTP fields
- **GIVEN** envelope `transport.protocol` is `sftp`
- **WHEN** event details are rendered
- **THEN** UI SHALL display SFTP/provider diagnostic fields
- **AND** UI SHALL NOT require HTTP-specific fields to render meaningful diagnostics

### Requirement: Web UI SHALL expose async-operation and partial-failure context
Web UI SHALL expose operation-level and partial-failure diagnostics when provided by the envelope.

#### Scenario: Async operation context is present
- **GIVEN** envelope context includes async operation metadata
- **WHEN** user inspects details
- **THEN** UI SHALL display operation id, current status, and next polling hint when available

#### Scenario: Partial failures are present
- **GIVEN** envelope context includes partial failure items
- **WHEN** user inspects details
- **THEN** UI SHALL render a per-item diagnostic list with resource id/path and error summary
