## ADDED Requirements

### Requirement: Targets Are Node-Scoped
Targets referenced by jobs SHALL be node-scoped. A job assigned to a node MUST only be able to reference targets available on that same node.

#### Scenario: Cross-node target selection is rejected
- **WHEN** a user attempts to create/update a job for node A referencing a target belonging to node B
- **THEN** the request is rejected

