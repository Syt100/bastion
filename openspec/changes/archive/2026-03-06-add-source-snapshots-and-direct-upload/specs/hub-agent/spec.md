## ADDED Requirements

### Requirement: Agent Creates Source Snapshot When Enabled
When snapshot mode requires snapshot creation, the agent execution path SHALL attempt to create the snapshot on the agent host and SHALL emit run events reflecting snapshot status.

#### Scenario: Snapshot lifecycle is visible
- **WHEN** a run starts packaging with snapshot mode enabled
- **THEN** run events include snapshot start/ready/unavailable states

