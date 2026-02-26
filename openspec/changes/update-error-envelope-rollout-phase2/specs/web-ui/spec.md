## ADDED Requirements

### Requirement: Web UI SHALL render envelope diagnostics in maintenance and snapshot management views
Maintenance and snapshot management diagnostic surfaces SHALL prioritize canonical envelope diagnostics when available.

#### Scenario: Task detail includes envelope diagnostics
- **GIVEN** task-related diagnostics include an event envelope
- **WHEN** user opens maintenance or snapshot task details
- **THEN** UI SHALL display envelope-based localized message/hint and key protocol details
- **AND** UI SHALL expose retriable and context metadata where available

### Requirement: Web UI SHALL preserve fallback compatibility for legacy task errors
UI SHALL continue to render meaningful diagnostics when canonical envelope fields are missing.

#### Scenario: Only legacy task error fields are available
- **GIVEN** task details provide `last_error_kind/last_error` without envelope
- **WHEN** user inspects the task diagnostics
- **THEN** UI SHALL render legacy diagnostics without regression
- **AND** generic localized fallback SHALL be shown if both envelope and legacy diagnostics are unavailable
