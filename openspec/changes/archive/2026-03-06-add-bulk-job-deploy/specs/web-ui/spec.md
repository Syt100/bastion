## ADDED Requirements

### Requirement: Jobs UI Provides “Deploy to Nodes” Action
The Web UI SHALL provide a “Deploy to nodes” action for an existing job.

#### Scenario: Operator starts deploy from job list
- **WHEN** the operator triggers “Deploy to nodes” for a job
- **THEN** the UI MUST open a flow to select target nodes and configure naming

### Requirement: UI Supports Label-based Selection and Naming Template
The deploy flow SHALL allow selecting nodes via labels (AND/OR) and SHALL provide a naming template input with a sensible default that includes the node id.

#### Scenario: Label selector targets a subset
- **WHEN** the operator selects labels and AND/OR mode
- **THEN** the UI MUST target the resolved node set

### Requirement: UI Shows Preview and Validation Results
The UI SHALL show a preview of planned job names and per-node validation results before executing the deploy.

#### Scenario: Preview highlights failures
- **GIVEN** some nodes are missing prerequisites
- **WHEN** the operator views the preview
- **THEN** the UI MUST highlight which nodes will fail and why

### Requirement: UI Shows Execution Results Via Bulk Operations Panel
The UI SHALL show deploy execution progress and per-node outcomes via the bulk operations panel/page.

#### Scenario: Operator can retry failed nodes
- **GIVEN** some nodes failed during deploy
- **WHEN** the operator chooses to retry failed
- **THEN** the UI MUST re-run only failed nodes via the bulk operations framework

