## ADDED Requirements

### Requirement: Preferred Node for Node-Scoped Navigation
The Web UI SHALL maintain a preferred node selection used as the default node when navigating to node-scoped pages from global routes.

#### Scenario: Selecting a node on a global page does not change the route
- **GIVEN** the user is on a global page (e.g. `/`)
- **WHEN** the user changes the node selector
- **THEN** the UI does not navigate away from the current page
- **AND** the preferred node is updated for subsequent node-scoped navigation

#### Scenario: Jobs navigation uses preferred node when not already in a node scope
- **GIVEN** the user is on a global page
- **AND** the preferred node is set to an agent id `agent1`
- **WHEN** the user navigates to Jobs
- **THEN** the UI navigates to `/n/agent1/jobs`

### Requirement: Node Context Cue in Page Headers
The Web UI SHALL display a clear node context cue on node-scoped pages.

#### Scenario: Jobs page shows node context
- **WHEN** the user opens `/n/:nodeId/jobs`
- **THEN** the page header shows the selected node context (Hub or Agent)

