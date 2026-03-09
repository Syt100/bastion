## ADDED Requirements

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

#### Scenario: Scheduled execution triggers a run
- **WHEN** the current time matches a job's cron schedule
- **THEN** the system starts a run for that job

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
