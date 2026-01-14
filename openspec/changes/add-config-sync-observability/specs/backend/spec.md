## ADDED Requirements

### Requirement: Hub Persists Desired and Applied Config Snapshot IDs Per Agent
The backend SHALL persist per-agent config synchronization state including the desired snapshot id and the last applied (acknowledged) snapshot id.

#### Scenario: Desired vs applied is recorded
- **GIVEN** an agent exists
- **WHEN** the Hub sends a config snapshot for that agent
- **THEN** the agent record MUST reflect the desired snapshot id
- **AND** the applied snapshot id MUST be updated only after a matching `ConfigAck`

### Requirement: ConfigAck Updates Applied Snapshot State
When the Hub receives an agent `ConfigAck`, the backend SHALL persist the acked snapshot id as the agent’s applied snapshot id along with a timestamp.

#### Scenario: Ack makes an agent “synced”
- **GIVEN** an agent has a desired snapshot id `S`
- **WHEN** the agent sends `ConfigAck` for snapshot `S`
- **THEN** the agent MUST be considered “synced” by the backend

### Requirement: Offline Nodes Are Supported Without Losing Desired State
If a node is offline when configuration changes, the backend SHALL still update the desired snapshot id and SHALL deliver the snapshot on reconnect.

#### Scenario: Offline node becomes pending then syncs on reconnect
- **GIVEN** an agent is offline
- **WHEN** a configuration change occurs that affects the agent
- **THEN** the backend MUST update the desired snapshot id and record that the node is pending delivery
- **AND** **WHEN** the agent reconnects
- **THEN** the Hub MUST send the desired snapshot

### Requirement: API Surfaces Config Sync Status
The backend SHALL expose sync state in authenticated agent list/detail APIs, including:
- desired snapshot id
- applied snapshot id
- sync status (synced/pending/error/offline)
- last sync attempt time and error summary (if any)

#### Scenario: Operator can view sync status via API
- **WHEN** the user lists agents
- **THEN** the response MUST include per-agent sync status fields

### Requirement: Operator Can Trigger “Sync Config Now” (Single and Bulk)
The backend SHALL provide an operator action to trigger sending the latest desired config snapshot to:
- a single agent, and
- multiple agents via bulk operations

#### Scenario: Sync now sends snapshot or records offline
- **WHEN** the operator triggers “sync now” for an online agent
- **THEN** the Hub MUST attempt to send the latest config snapshot
- **AND** **WHEN** the operator triggers “sync now” for an offline agent
- **THEN** the system MUST record the node as pending delivery without failing the entire operation

