## MODIFIED Requirements

### Requirement: UI Exposes Snapshot Controls and Status
The Web UI SHALL:
- expose snapshot mode/provider controls in the filesystem job editor
- show snapshot status/provider in run detail when configured

#### Scenario: User configures snapshot mode
- **WHEN** a user edits a filesystem job
- **THEN** snapshot mode can be set to off/auto/required

#### Scenario: Run detail shows snapshot status
- **WHEN** a run has snapshot status in summary/events
- **THEN** the UI shows the snapshot provider and outcome

