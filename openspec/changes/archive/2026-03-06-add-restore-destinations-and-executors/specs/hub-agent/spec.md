## ADDED Requirements

### Requirement: Restore Tasks Are Persisted for Reconnect-Safe Execution
The Hub SHALL persist restore tasks dispatched to Agents and SHALL support reconnect-safe delivery and idempotency.

#### Scenario: Agent reconnects during a restore
- **GIVEN** an Agent disconnects and reconnects during an Agent-executed restore
- **WHEN** the Agent reconnects
- **THEN** the Hub can re-deliver the restore task without duplicating the operation or corrupting the destination

### Requirement: Hub Acts as a Relay for Cross-Node Restore Data Movement
The Hub SHALL support relaying artifact bytes between nodes so that restore can work across combinations of storage backends and executor nodes.

#### Scenario: Restore from Agent-local target to another Agent-local destination
- **GIVEN** a run’s artifacts are stored in a LocalDir on Agent A
- **AND** the user restores to a LocalDir destination on Agent B
- **WHEN** the restore is started
- **THEN** the Hub relays artifact bytes from Agent A to Agent B over their respective Hub connections

