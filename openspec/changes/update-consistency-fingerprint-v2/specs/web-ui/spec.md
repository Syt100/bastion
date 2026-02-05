## MODIFIED Requirements

### Requirement: UI Renders v2 Consistency Evidence
The Web UI SHALL render the v2 consistency report:
- totals + breakdown
- sample paths and reasons
- (optional) expose `after_handle` evidence in a details view

#### Scenario: UI shows v2 sample evidence
- **WHEN** a run summary contains a v2 consistency report
- **THEN** the UI shows sample paths and reasons
- **AND** the UI can show before/after fingerprints for troubleshooting

