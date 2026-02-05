## MODIFIED Requirements

### Requirement: Builders Persist Consistency Report v2
Backup builders SHALL persist the v2 consistency report in run summary and run events when warnings are present.

#### Scenario: Summary stores v2 consistency report
- **WHEN** a backup completes and warnings are detected
- **THEN** the run summary contains a v2 consistency report with totals and samples (bounded)

