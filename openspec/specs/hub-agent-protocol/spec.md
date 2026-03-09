# hub-agent-protocol Specification

## Purpose
TBD - created by archiving change add-graceful-run-operation-cancel. Update Purpose after archive.
## Requirements
### Requirement: Hub Can Dispatch Cancel Requests For Agent Tasks
The Hub↔Agent protocol SHALL support explicit cancel messages for in-flight run tasks and operation tasks.

#### Scenario: Hub requests cancellation of agent run task
- **GIVEN** an agent is executing run task `<run_id>`
- **WHEN** the Hub issues a run cancel request
- **THEN** the agent receives a cancel message for `<run_id>` and starts cooperative stop handling

#### Scenario: Hub requests cancellation of agent operation task
- **GIVEN** an agent is executing operation task `<op_id>`
- **WHEN** the Hub issues an operation cancel request
- **THEN** the agent receives a cancel message for `<op_id>` and starts cooperative stop handling

### Requirement: Agent Reports Cancel-Aware Completion
The Hub↔Agent protocol SHALL allow agent task completion to represent canceled outcomes.

#### Scenario: Agent finishes canceled run
- **GIVEN** an agent run task received a cancel request
- **WHEN** the agent reaches a cancellation checkpoint and exits
- **THEN** task completion to Hub indicates canceled outcome semantics

### Requirement: Cancel Delivery Is Reconnect-Safe
Cancel intent for agent-executed tasks SHALL remain effective across transient Hub↔Agent reconnects.

#### Scenario: Agent reconnects after cancel requested
- **GIVEN** Hub has recorded cancel intent for an in-flight agent task
- **AND** the agent disconnects and reconnects before finishing
- **WHEN** synchronization resumes
- **THEN** the agent still receives/observes cancel intent for that task

### Requirement: Agent Supports WebDAV Directory Listing for Node-Scoped API
The Hub↔Agent protocol SHALL support a node-scoped WebDAV list request so that the Hub can browse WebDAV paths from the Agent’s network vantage point.

#### Scenario: Hub forwards a WebDAV list request to an Agent
- **GIVEN** an Agent is connected
- **WHEN** the Hub forwards a WebDAV list request for `node_id=<agent_id>`
- **THEN** the Agent performs the PROPFIND and returns entries and paging cursor

### Requirement: Secrets Snapshot Can Include Backup Age Identities
The Hub→Agent secrets snapshot protocol SHALL support distributing backup age identities (`backup_age_identity`) to Agents.

#### Scenario: Agent receives an age identity for restore
- **GIVEN** an encrypted run references key name `K`
- **WHEN** the Hub distributes `backup_age_identity/K` to an Agent and sends an updated secrets snapshot
- **THEN** the Agent persists the key and can decrypt restore payloads that require `K`

### Requirement: Agent FS List Supports Sorting
The Hub↔Agent filesystem list protocol SHALL support sorting directory entries to enable consistent UX when browsing large directories.

#### Scenario: Hub requests a sorted page
- **GIVEN** an Agent is connected
- **WHEN** the Hub requests a filesystem list page with `sort_by` and `sort_dir`
- **THEN** the Agent returns entries ordered by the requested sort
- **AND** the paging cursor remains stable for that sort order

