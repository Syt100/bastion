## MODIFIED Requirements

### Requirement: Run History Retention Must Not Orphan Snapshots
The system SHALL prune old run history based on retention settings, but SHALL NOT prune runs that still have an existing snapshot.

#### Scenario: Keep runs for live snapshots
- **GIVEN** a run ended before the retention cutoff
- **AND** the run has a snapshot record with status `present`
- **WHEN** run retention pruning executes
- **THEN** the run is not deleted

#### Scenario: Prune runs after snapshot deletion
- **GIVEN** a run ended before the retention cutoff
- **AND** the run has a snapshot record with status `deleted` (or `missing`)
- **WHEN** run retention pruning executes
- **THEN** the run can be deleted

