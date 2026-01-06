## ADDED Requirements

### Requirement: Offline-Executed Runs Remain Understandable
When runs are executed on an Agent while the Hub is unreachable and later synced, the Web UI SHALL keep the user experience understandable and consistent.

The UI MAY annotate runs as “executed offline” and/or show delayed ingestion time.

#### Scenario: User can distinguish delayed ingestion
- **WHEN** an offline-executed run is synced later to the Hub
- **THEN** the UI can indicate that the run executed while offline (optional) without breaking run viewing workflows
