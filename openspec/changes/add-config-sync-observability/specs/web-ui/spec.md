## ADDED Requirements

### Requirement: Agents Page Shows Config Sync Status
The Web UI SHALL show a quick, scannable config sync status indicator for each agent.

#### Scenario: Operator can identify out-of-sync nodes
- **WHEN** the user opens the Agents page
- **THEN** each agent row MUST display whether the agent is synced, pending, offline, or in error

### Requirement: Agent Details Show Desired/Applied Snapshot and Errors
The Web UI SHALL provide a detailed view that shows desired/applied snapshot ids and the latest sync error information.

#### Scenario: Operator inspects a node’s sync details
- **WHEN** the user opens an agent detail view
- **THEN** the UI MUST display desired/applied snapshot ids and last sync error (if any)

### Requirement: UI Supports “Sync Now” Actions
The Web UI SHALL provide “sync now” actions for single-node and bulk selections (via bulk operations UI).

#### Scenario: User triggers single-node sync now
- **WHEN** the user clicks “sync now” for a node
- **THEN** the UI MUST call the backend API and display success/error feedback

