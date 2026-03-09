## MODIFIED Requirements

### Requirement: Run Events Use v2 Consistency Report
When emitting `source_consistency` run events, the system SHALL include the v2 report structure in event fields.

#### Scenario: Run event includes v2 fields
- **WHEN** a run emits a `source_consistency` event
- **THEN** the event fields include v2 sample entries with `before`, `after_handle`, and `after_path` when available

