## ADDED Requirements

### Requirement: Persist Run Stage Boundaries From Progress Snapshots
The backend SHALL persist stage boundaries for progress snapshots as run events.

#### Scenario: Progress snapshot stage transition emits a run event
- **GIVEN** a run receives progress snapshots with stage values
- **WHEN** the stage value changes (e.g. `scan` → `packaging` → `upload`)
- **THEN** the backend records a run event for the new stage

#### Scenario: Identical stage snapshots do not emit duplicate run events
- **GIVEN** a run receives progress snapshots with a stage value
- **WHEN** multiple snapshots arrive with the same stage value
- **THEN** the backend does not record duplicate stage events for that stage

