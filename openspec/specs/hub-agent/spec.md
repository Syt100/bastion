# hub-agent Specification

## Purpose
TBD - created by archiving change refactor-backup-source-target-driver-architecture. Update Purpose after archive.
## Requirements
### Requirement: Hub and Agent MUST Exchange Driver Capability Metadata
Hub-Agent protocol payloads MUST include driver identifiers and capability metadata needed for
planning and compatibility checks.

#### Scenario: Agent advertises installed driver capabilities
- **WHEN** an agent connects or refreshes config snapshot state
- **THEN** agent exposes source/target driver identifiers and capability metadata to Hub

### Requirement: Hub MUST Enforce Capability-Aware Dispatch
Hub MUST dispatch runs to an agent only when the agent can satisfy required source/target driver
and protocol capabilities.

#### Scenario: Dispatch blocked by missing target driver
- **WHEN** a job requires a target driver that the selected agent does not provide
- **THEN** dispatch is rejected with a structured capability mismatch error

### Requirement: Protocol Evolution MUST Support Mixed-Version Rollouts
The protocol MUST support mixed Hub-Agent versions during migration from enum-based specs to
driver-envelope specs.

#### Scenario: New Hub with old Agent
- **WHEN** Hub supports driver-envelope metadata and agent only supports legacy fields
- **THEN** Hub uses compatibility mode to preserve run execution without data loss

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

### Requirement: Hub Can Distribute Backup Age Identities to Selected Agents
The Hub SHALL support copying a backup age identity secret from Hub scope to a selected Agent scope to enable Agent-executed encrypted restores.

#### Scenario: Hub distributes a missing key on-demand
- **GIVEN** a restore is requested to execute on Agent `<agent_id>`
- **AND** the run references age key name `K`
- **AND** Agent `<agent_id>` does not have `backup_age_identity/K`
- **WHEN** the restore is started
- **THEN** the Hub copies the key to Agent scope and refreshes secrets delivery before the restore begins

### Requirement: Deterministic Config Snapshot IDs
The Hub SHALL compute `ConfigSnapshot.snapshot_id` deterministically from the snapshot content so identical snapshots yield identical IDs.

#### Scenario: Same snapshot content yields same snapshot_id
- **WHEN** the Hub computes two configuration snapshots with identical job content for the same Agent
- **THEN** the two snapshots have the same `snapshot_id`

### Requirement: Hub Avoids Re-sending Unchanged Snapshots
The Hub SHALL avoid re-sending an unchanged configuration snapshot to a connected Agent.

#### Scenario: Unchanged snapshot is not re-sent
- **WHEN** a configuration change handler is triggered but the computed snapshot content is unchanged
- **THEN** the Hub does not send a redundant `ConfigSnapshot` to the Agent

### Requirement: Agent Offline Persistence Avoids Blocking the Async Runtime
When persisting offline run state and events, the Agent SHALL avoid blocking the async runtime hot path (e.g., by using async I/O or a dedicated writer task).

#### Scenario: Offline run event persistence does not block scheduling
- **WHEN** an offline run emits run events rapidly
- **THEN** the Agent persists events without blocking the async scheduler/task execution loop

### Requirement: Hub Stores Node-Scoped Configuration
The Hub SHALL store node-scoped target/credential configuration for the Hub node and for Agent nodes.

#### Scenario: Agent has isolated targets
- **WHEN** two Agents are enrolled (A and B)
- **THEN** targets created for Agent A are not visible/usable for Agent B

### Requirement: Managed Agent Configuration Is Read-Only Locally
When an Agent is enrolled and managed by a Hub, any local configuration UI (if enabled) SHALL be read-only for configuration changes and MUST direct users to manage configuration in the Hub UI.

#### Scenario: Agent config UI is read-only
- **WHEN** a user opens the Agent local UI while it is enrolled
- **THEN** configuration editors are disabled and the UI indicates management is done in the Hub

### Requirement: Agent Reconnect Uses Jittered Backoff
When reconnecting to the Hub, Agents SHALL use backoff with jitter to avoid synchronized reconnect storms.

#### Scenario: Reconnect includes jitter
- **WHEN** an Agent reconnects repeatedly due to network instability
- **THEN** the retry delay includes jitter so multiple agents do not reconnect at the exact same cadence

### Requirement: Agent Heartbeat Has a Pong Timeout
Agents SHALL use heartbeat ping/pong and SHALL reconnect when a pong is not observed within a configured timeout window.

#### Scenario: Missed pong triggers reconnect
- **WHEN** the Agent does not observe a pong within the timeout window
- **THEN** it reconnects to the Hub

### Requirement: Task ACK and Retry Boundaries Reduce Duplicate Work
The Hub/Agent protocol SHALL define clear ACK and retry boundaries so that reconnect-driven re-delivery does not cause excessive duplicate work.

#### Scenario: Duplicate task deliveries are handled safely
- **WHEN** a task is delivered more than once due to reconnect
- **THEN** the Agent handles the duplicate delivery without executing the task multiple times when avoidable

### Requirement: Enrollment Tokens
The Hub SHALL provide enrollment tokens with a configurable expiration time and SHALL allow limiting token usage.

#### Scenario: Expired token is rejected
- **WHEN** an Agent attempts to enroll using an expired token
- **THEN** enrollment fails

### Requirement: Agent Registration and Long-Lived Key
The Hub SHALL exchange a valid enrollment token for an `agent_id` and long-lived `agent_key`, and the Hub SHALL store only a verifier (hash) of the `agent_key`.

#### Scenario: Agent is issued a key
- **WHEN** an Agent enrolls with a valid token
- **THEN** it receives an `agent_id` and `agent_key` and can connect without the enrollment token

### Requirement: Agent-Initiated WebSocket Connection
Agents SHALL initiate a WebSocket connection to the Hub and SHALL send a `hello` message containing version and capability information.

#### Scenario: Hub records agent capabilities
- **WHEN** an Agent connects and sends `hello`
- **THEN** the Hub records the Agent as online with its declared capabilities

### Requirement: Task Dispatch and Acknowledgement
The Hub SHALL dispatch tasks to Agents and Agents SHALL acknowledge receipt, enabling reconnect-safe delivery.

#### Scenario: Task is acknowledged
- **WHEN** the Hub sends a task with a `task_id`
- **THEN** the Agent sends an acknowledgement for that `task_id`

### Requirement: Agent Key Revocation
The Hub SHALL support revoking an Agent key and SHALL reject subsequent connections using the revoked key.

#### Scenario: Revoked Agent cannot connect
- **WHEN** an Agent key is revoked
- **THEN** the Agent's next connection attempt is rejected

### Requirement: Agent Key Rotation
The Hub SHALL support rotating an Agent key by issuing a new `agent_key` for an existing `agent_id` and invalidating the old key.

#### Scenario: Old key becomes invalid after rotation
- **WHEN** an Agent key is rotated
- **THEN** the old key can no longer connect and only the new key is accepted

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

