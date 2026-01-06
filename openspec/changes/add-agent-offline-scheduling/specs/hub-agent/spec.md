## ADDED Requirements

### Requirement: Hub-to-Agent Configuration Sync
The Hub SHALL provide a mechanism to sync per-Agent configuration snapshots (jobs + targets + required credentials) to enrolled Agents.

Agents MUST persist the last-known snapshot locally.

#### Scenario: Agent caches config for offline use
- **WHEN** an Agent is enrolled and the Hub has jobs assigned to that Agent
- **THEN** the Hub sends a configuration snapshot to the Agent
- **AND** the Agent persists it locally for offline execution

### Requirement: Agent Syncs Runs Back to Hub
Agents SHALL persist run outcomes and events locally and SHALL sync missing run records and events back to the Hub when connected.

The Hub MUST deduplicate ingest to avoid double-recording runs.

#### Scenario: Offline run appears in Hub after reconnect
- **WHEN** an Agent executes a run while offline and later reconnects to the Hub
- **THEN** the Hub receives the run record and events
- **AND** the run becomes visible in the Hub UI

