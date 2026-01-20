## ADDED Requirements

### Requirement: Hub Dispatches Restore Tasks to Agents
The Hub↔Agent protocol SHALL support dispatching restore tasks to Agents with reconnect-safe delivery semantics.

#### Scenario: Agent receives a restore task
- **GIVEN** an Agent is connected
- **WHEN** the Hub dispatches a restore task for operation `<op_id>`
- **THEN** the Agent acknowledges receipt and begins emitting operation events

### Requirement: Agent Reports Restore Operation Events and Completion
The Hub↔Agent protocol SHALL support operation event streaming and completion reporting from Agent to Hub for long-running restore tasks.

#### Scenario: UI can display restore progress for Agent-executed restore
- **WHEN** an Agent-executed restore emits progress events
- **THEN** the Hub persists those events and the Web UI can display them via existing operation event APIs

### Requirement: Artifact Byte Streams Can Be Relayed via the Hub
The Hub↔Agent protocol SHALL support streaming run artifact bytes between nodes with flow control so that the Hub can act as a relay.

#### Scenario: Agent pulls artifact bytes via Hub relay
- **GIVEN** a restore executor Agent needs to read a payload part stored on another backend
- **WHEN** the Agent requests an artifact byte stream via the Hub
- **THEN** the Hub streams the bytes with bounded memory usage on both ends

