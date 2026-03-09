# backup-jobs Specification

## Purpose
TBD - created by archiving change refactor-backup-source-target-driver-architecture. Update Purpose after archive.
## Requirements
### Requirement: Job Spec MUST Support Driver Envelopes
Backup job specs MUST support a canonical driver envelope for both source and target definitions.
The envelope MUST include `type`, `version`, and driver-specific `config`. Target envelopes MUST
also support explicit `auth_refs`.

#### Scenario: Persist and read canonical V2 driver envelopes
- **WHEN** a user saves a backup job using a source driver and target driver
- **THEN** the stored canonical spec contains `source.type`, `source.version`, `source.config`
- **AND** contains `target.type`, `target.version`, `target.config`, `target.auth_refs`

### Requirement: Job Spec V1 MUST Remain Backward Compatible
The system MUST accept existing V1 job specs and translate them to canonical V2 representation
without changing runtime behavior.

#### Scenario: Execute existing V1 job with V2 runtime
- **WHEN** a legacy V1 filesystem/sqlite/vaultwarden job is loaded
- **THEN** the system translates it to canonical V2 in-memory
- **AND** execution semantics remain equivalent to the pre-migration behavior

### Requirement: Job Spec MUST Keep Credentials Out of Driver Config
Credential material MUST NOT be stored inline in source or target driver configs. Credentials MUST
be referenced via secret references and resolved at runtime in node scope.

#### Scenario: Reject inline target credentials
- **WHEN** a job payload includes direct credential fields inside target config
- **THEN** validation fails with a structured error indicating that secret references are required

### Requirement: Targets Are Node-Scoped
Targets referenced by jobs SHALL be node-scoped. A job assigned to a node MUST only be able to reference targets available on that same node.

#### Scenario: Cross-node target selection is rejected
- **WHEN** a user attempts to create/update a job for node A referencing a target belonging to node B
- **THEN** the request is rejected

### Requirement: Per-Job Notification Override (Inherit or Custom)
Each job SHALL support notification configuration with two modes:
- `inherit`: use global/channel settings and all enabled destinations
- `custom`: per channel, choose destinations explicitly (multi-select)

Disabled destinations MAY be selected in `custom` mode, but SHALL be treated as not deliverable until enabled.

#### Scenario: Job inherits global notifications
- **WHEN** a job is created without an explicit notification override
- **THEN** the job inherits global notification settings

#### Scenario: Job uses a custom destination subset
- **WHEN** a job is configured to use WeCom destinations `A,B` and no Email destinations
- **THEN** only `A,B` receive WeCom notifications on run completion
- **AND** no Email notifications are sent for that job

### Requirement: Job Types
The system SHALL support defining jobs for file backups and SQLite backups, and SHALL support application recipes that compose these primitives (e.g., Vaultwarden).

#### Scenario: Create a file backup job
- **WHEN** a user creates a job with a filesystem source and WebDAV target
- **THEN** the job is stored and becomes runnable

### Requirement: Optional Encryption Per Job
The system SHALL allow configuring optional encryption per job and SHALL default to no encryption when not specified.

#### Scenario: Default encryption is disabled
- **WHEN** a job is created without an explicit encryption configuration
- **THEN** runs for that job execute with encryption disabled

#### Scenario: Age encryption is enabled
- **WHEN** a job is configured with age encryption and key name `K`
- **THEN** runs for that job execute with age encryption and record `K` in the manifest

### Requirement: Execution Node (Hub or Agent)
The system SHALL allow selecting an execution node for a job: the local Hub node or a specific enrolled Agent node.

#### Scenario: Create an Agent job
- **WHEN** a user creates a job and assigns it to an Agent
- **THEN** the job is stored with an `agent_id` and is runnable on that Agent

#### Scenario: Invalid Agent is rejected
- **WHEN** a user creates or updates a job with an `agent_id` that does not exist or is revoked
- **THEN** the request is rejected

### Requirement: Agent-Executed Runs
For jobs assigned to an Agent, the Hub SHALL dispatch run execution to that Agent and SHALL record run events and final status in the Hub database.

#### Scenario: Run executes on Agent
- **WHEN** a user triggers a run for an Agent-assigned job
- **THEN** the Hub dispatches a task to the Agent and the run completes with events visible in the UI

### Requirement: Agent Receives Resolved Encryption Parameters
For encrypted jobs executed on an Agent, the Hub SHALL provide the Agent sufficient resolved encryption parameters to produce the encrypted artifact stream (e.g., an age recipient).

#### Scenario: Hub sends age recipient
- **WHEN** an Agent-assigned job runs with age encryption enabled using key name `K`
- **THEN** the dispatched task includes the age recipient and key name `K`

### Requirement: Built-In Scheduler
The system SHALL provide a built-in scheduler to execute jobs based on cron expressions.

For jobs assigned to an Agent node, the Agent SHOULD continue scheduled execution using its cached schedule when the Hub is unreachable.

#### Scenario: Agent runs scheduled job while Hub is unreachable
- **WHEN** the Hub is unreachable for an enrolled Agent and a cached job schedule matches the current time
- **THEN** the Agent starts a run locally
- **AND** the run is synced back to the Hub when connectivity returns

### Requirement: Overlap Policy (Reject or Queue)
The system SHALL support per-job overlap policies: reject concurrent executions or queue them, and SHALL NOT run overlapping executions in parallel.

#### Scenario: Reject overlap
- **WHEN** a job is already running and overlap policy is set to reject
- **THEN** a new scheduled run is rejected and recorded as such

### Requirement: Run History and Logs in SQLite
The system SHALL store run history and structured events/logs in SQLite and SHALL support a default retention of 180 days, configurable by the user.

#### Scenario: Old runs are deleted
- **WHEN** the retention period elapses
- **THEN** runs and their events older than the retention are removed

