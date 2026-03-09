## ADDED Requirements

### Requirement: Bulk Job Deploy Clones a Source Job to Target Nodes
The backend SHALL support a bulk job deploy operation that clones an existing source job to multiple target nodes selected by explicit ids or label selectors.

#### Scenario: Jobs are created for each target node
- **GIVEN** a source job exists
- **WHEN** the operator deploys the job to multiple target nodes
- **THEN** the backend MUST create a corresponding job for each targeted node

### Requirement: Name Template Default and Collision Handling
Bulk job deploy SHALL use a name template with a default that includes the node id (e.g., `"{name} ({node})"`). If a generated name still collides, the backend SHALL automatically disambiguate with a suffix.

#### Scenario: Default name includes node id
- **WHEN** the operator does not override the name template
- **THEN** each deployed job name MUST include the target node id

#### Scenario: Collision is auto-suffixed
- **GIVEN** a generated job name already exists on a node
- **WHEN** the deploy operation attempts to create the job
- **THEN** the backend MUST adjust the name to avoid ambiguity (e.g., add `#2`)

### Requirement: Per-node Preflight Validation and Clear Errors
The backend SHALL validate prerequisites per node (e.g., node-scoped secrets referenced by the job spec) and SHALL record failures with clear, actionable error summaries.

#### Scenario: Missing credential fails only that node
- **GIVEN** the source job references a WebDAV credential name
- **AND** a target node does not have that credential
- **WHEN** the deploy operation runs
- **THEN** that node MUST fail with a clear error message
- **AND** other nodes MUST continue processing

### Requirement: Preview Before Execution
The backend SHALL support a preview capability that returns, per node, the planned job name and validation outcome prior to execution.

#### Scenario: Preview returns planned names and validation
- **WHEN** the operator requests a deploy preview
- **THEN** the preview MUST include planned names and per-node validation results

