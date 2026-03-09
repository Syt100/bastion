## ADDED Requirements

### Requirement: Run events UI highlights actionable failure diagnostics
The Web UI SHALL prominently present actionable diagnostics from failed run events without requiring users to inspect raw JSON.

#### Scenario: Failed event includes operator hint
- **GIVEN** a run failed event contains `hint` and classification fields
- **WHEN** user opens run events
- **THEN** UI displays hint/diagnostic cues in list chips or equivalent compact markers
- **AND** raw JSON details remain available in the event detail panel

### Requirement: Run events UI surfaces transport metadata for upload failures
The Web UI SHALL surface key transport metadata for upload-related failures when available.

#### Scenario: Upload failure includes HTTP and part metadata
- **GIVEN** failed event fields include HTTP status and part size/name
- **WHEN** user inspects the run events list/details
- **THEN** UI displays recognizable status/part diagnostics to speed troubleshooting
