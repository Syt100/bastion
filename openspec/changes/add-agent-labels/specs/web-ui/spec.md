## ADDED Requirements

### Requirement: Agents Page Displays Labels
The Web UI SHALL display agent labels as tags/chips on the Agents page.

#### Scenario: Labels are visible on the agent list
- **WHEN** the user opens the Agents page
- **THEN** each agent row MUST display its labels (if any)

### Requirement: Agents Page Supports Label Filtering (AND/OR)
The Web UI SHALL provide a label filter supporting multi-select labels and an AND/OR toggle.

#### Scenario: AND mode filter narrows results
- **GIVEN** there are agents with different label sets
- **WHEN** the user selects multiple labels with AND mode
- **THEN** the list MUST show only agents that contain all selected labels

#### Scenario: OR mode filter broadens results
- **GIVEN** there are agents with different label sets
- **WHEN** the user selects multiple labels with OR mode
- **THEN** the list MUST show agents that contain any selected label

### Requirement: Agent Label Editor
The Web UI SHALL allow a user to add/remove labels on a single agent.

#### Scenario: User edits labels for one agent
- **WHEN** the user opens an agent’s label editor and updates labels
- **THEN** the UI MUST persist the changes via the API
- **AND** refresh the agent list to reflect the updated labels

