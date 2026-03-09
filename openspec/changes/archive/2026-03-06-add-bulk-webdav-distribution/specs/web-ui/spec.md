## ADDED Requirements

### Requirement: Hub Storage UI Provides “Distribute to Nodes” Flow
The Web UI SHALL provide a Hub-context flow to distribute an existing WebDAV credential to selected nodes.

#### Scenario: Operator selects credential and targets
- **WHEN** the operator opens the distribution flow
- **THEN** the UI MUST allow selecting a credential and a target selector (labels or explicit nodes)

### Requirement: UI Shows Preview Before Execute
The Web UI SHALL display a preview list indicating for each node whether it will be skipped or updated, and SHALL allow enabling overwrite explicitly.

#### Scenario: Preview indicates skip by default
- **GIVEN** a destination credential exists on some nodes
- **WHEN** the operator views the preview with overwrite disabled
- **THEN** the UI MUST indicate those nodes will be skipped

### Requirement: UI Shows Execution Results Via Bulk Operations Panel
The Web UI SHALL reuse the bulk operations panel/page to show execution progress and per-node errors for WebDAV distribution.

#### Scenario: Operator can inspect failed nodes
- **WHEN** the bulk operation completes with failures
- **THEN** the UI MUST show which nodes failed and the error summary per node

