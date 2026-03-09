## ADDED Requirements

### Requirement: Local Directory Targets Are Node-Scoped
Local directory targets SHALL be node-scoped and SHALL be configurable per node.

#### Scenario: Local target differs per node
- **WHEN** node A and node B each configure a local directory target
- **THEN** the configured base directories may differ and remain isolated per node

