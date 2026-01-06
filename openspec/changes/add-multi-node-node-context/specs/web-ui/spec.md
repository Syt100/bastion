## ADDED Requirements

### Requirement: Node Context and Node-Scoped Routes
The Web UI SHALL support a first-class node context and SHALL encode it in node-scoped routes under `/n/:nodeId/**` so node context persists across refreshes and deep links.

`nodeId` MUST support:
- the local Hub node (reserved id `hub`), and
- enrolled Agent nodes (using their `agent_id`).

#### Scenario: Node context survives refresh
- **WHEN** a user opens `/n/hub/jobs` and refreshes the page
- **THEN** the UI remains in the Hub node context and the Jobs list is shown for the Hub node

### Requirement: Node Switcher
The Web UI SHALL provide a node switcher that allows selecting the Hub node and any enrolled Agent node.

The node switcher SHOULD display basic status (e.g. online/offline) for Agents.

#### Scenario: User switches to an Agent node
- **WHEN** a user selects an Agent node from the node switcher
- **THEN** the UI navigates to the equivalent node-scoped route under `/n/:nodeId/**`

### Requirement: Per-Node UX Matches Single-Node Behavior
In node context, the Web UI SHALL behave like a single-node app for the selected node:
- lists are filtered to the selected node,
- create/edit defaults to the selected node,
- cross-node selection controls are hidden or disabled.

#### Scenario: Create job defaults to selected node
- **WHEN** the user is on `/n/<agent_id>/jobs` and opens the Create Job wizard
- **THEN** the job is created for that Agent node without requiring a separate node selection step

### Requirement: Global Pages Are Clearly Separated
The Web UI SHALL keep global management pages outside node-scoped routes (e.g. Agents management, global notifications/settings).

#### Scenario: Agents page is global
- **WHEN** the user opens the Agents page
- **THEN** it shows all enrolled Agents regardless of the current node context

