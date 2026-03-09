## MODIFIED Requirements

### Requirement: Run Summary Includes Snapshot Status
When snapshot mode is configured for a run, the run summary SHALL include snapshot status information (mode/provider/status/reason).

#### Scenario: Summary records snapshot provider
- **WHEN** a snapshot is successfully created
- **THEN** the run summary contains the provider name and status

