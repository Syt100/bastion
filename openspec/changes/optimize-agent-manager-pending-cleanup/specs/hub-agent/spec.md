## ADDED Requirements

### Requirement: Pending Hub-to-Agent Requests Are Completed On Disconnect
When an Agent disconnects, the Hub SHALL complete and remove all pending Hub-to-Agent requests associated with that Agent so callers do not hang and the Hub does not retain stale state.

#### Scenario: Pending list requests fail fast on disconnect
- **GIVEN** the Hub has a pending list request for an Agent (filesystem or WebDAV)
- **WHEN** the Agent disconnects
- **THEN** the pending request completes with an explicit "agent disconnected/offline" error
- **AND** the pending request slot is removed from in-memory state

#### Scenario: Pending artifact stream open fails with an explicit error
- **GIVEN** the Hub has a pending artifact stream open request for an Agent
- **WHEN** the Agent disconnects
- **THEN** the request completes with a structured error (not an ambiguous "channel closed")

