## ADDED Requirements

### Requirement: Targets UI Is Node-Scoped
In node context, the Web UI SHALL render Targets/Storage pages for the selected node and SHALL only show and edit targets within that node scope.

#### Scenario: Storage page shows only node targets
- **WHEN** the user opens `/n/<agent_id>/settings/storage`
- **THEN** only targets belonging to that Agent node are shown

### Requirement: Job Editor Enforces Node-Scoped Targets
In node context, the Web UI SHALL only allow selecting targets belonging to the selected node when creating/editing jobs.

#### Scenario: Job editor hides cross-node targets
- **WHEN** the user edits a job on `/n/hub/jobs`
- **THEN** the target selector shows only Hub-scoped targets

