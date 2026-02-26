## ADDED Requirements

### Requirement: Backend SHALL emit canonical envelopes for remaining bridged and execute-stage failures
Backend failure diagnostics that are bridged from Agent messages or produced during execute-stage processing SHALL include canonical `error_envelope` fields in event payloads.

#### Scenario: Agent snapshot-delete failure is bridged with canonical envelope
- **GIVEN** Hub receives a snapshot delete failure/result from an Agent
- **WHEN** backend appends task/run events for that failure
- **THEN** event fields SHALL include `error_envelope` with stable `schema_version`, `code`, `kind`, `retriable`, and `transport.protocol`
- **AND** legacy fields SHALL remain present during migration

#### Scenario: Execute-stage failure/warn event includes canonical envelope
- **GIVEN** an execute-stage path (filesystem/sqlite/vaultwarden) emits a warning or failure event
- **WHEN** backend appends the event
- **THEN** fields SHALL include `error_envelope` with stable code namespace and origin metadata
- **AND** transport metadata SHALL reflect the best-known protocol context when available

### Requirement: Backend SHALL synthesize envelopes when upstream payload lacks canonical diagnostics
When upstream/bridged payloads do not include canonical envelope fields, backend SHALL synthesize a valid envelope from available structured context.

#### Scenario: Agent result lacks envelope but has status/error_kind
- **GIVEN** a bridged Agent result includes legacy status/error fields but no envelope
- **WHEN** backend persists the corresponding event
- **THEN** backend SHALL synthesize `error_envelope` using stable fallback mapping
- **AND** synthesized envelope SHALL preserve retriable and error-kind semantics when inferable

### Requirement: Rollout SHALL remain backward-compatible for existing clients
Migration in this phase SHALL preserve existing readers that still consume legacy fields.

#### Scenario: Legacy event readers continue to function
- **GIVEN** a client that only reads legacy diagnostic fields
- **WHEN** backend emits the new envelope fields
- **THEN** legacy fields SHALL remain available with unchanged compatibility behavior
