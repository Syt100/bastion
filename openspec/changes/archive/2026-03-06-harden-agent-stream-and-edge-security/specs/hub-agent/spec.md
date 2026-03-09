## ADDED Requirements

### Requirement: Hub Artifact Stream Requests Are Bound To Authorized Agent Tasks
The Hub SHALL authorize each incoming agent-initiated artifact stream open request against an operation/task context that belongs to the authenticated agent.

#### Scenario: Agent cannot open stream for another agent task
- **GIVEN** agent A is authenticated and agent B owns an open task/operation for run R
- **WHEN** agent A requests `ArtifactStreamOpen` with the `op_id` and `run_id` of agent B
- **THEN** the Hub rejects the request
- **AND** no stream is opened

#### Scenario: Agent can open stream for its own authorized task
- **GIVEN** an authenticated agent owns the referenced open task/operation and matching run
- **WHEN** the agent requests `ArtifactStreamOpen`
- **THEN** the Hub opens the stream and returns stream metadata

### Requirement: Hub Emits Security Audit Logs For Stream Authorization Denials
The Hub SHALL emit structured warning logs for denied artifact stream open requests with enough context for security auditing.

#### Scenario: Denied stream open is logged
- **WHEN** the Hub rejects an `ArtifactStreamOpen` request due to authorization mismatch
- **THEN** the Hub logs the authenticated agent id, requested op_id, requested run_id, and deny reason
