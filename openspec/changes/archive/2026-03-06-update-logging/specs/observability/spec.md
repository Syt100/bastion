## ADDED Requirements

### Requirement: Default Backend Logs Are Visible
The system SHALL initialize backend logging such that `INFO` logs are visible by default when no explicit log filter is provided.

#### Scenario: Default startup shows INFO logs
- **WHEN** the service starts without `RUST_LOG`, `BASTION_LOG`, or `--log`
- **THEN** `INFO` logs from Bastion are emitted to the console

### Requirement: Log Filtering is Configurable
The system SHALL allow configuring the log filter via CLI and/or environment variables.

#### Scenario: Operator increases verbosity
- **WHEN** the service starts with `--log debug` (or an equivalent environment variable)
- **THEN** `DEBUG` logs are emitted according to the configured filter

### Requirement: Optional File Logging with Rotation
The system SHALL support optional log output to a file, and SHALL support time-based rotation for that file output.

#### Scenario: File logging is enabled
- **WHEN** the service starts with `--log-file /var/log/bastion/bastion.log`
- **THEN** logs are written to both the console and the configured file destination

#### Scenario: File logging rotates
- **WHEN** file logging is enabled with daily rotation
- **THEN** the system writes new log files per day in the configured location

### Requirement: Rotated Log Retention is Configurable
When log rotation is enabled, the system SHALL provide a configurable retention policy for rotated log files.

#### Scenario: Old rotated logs are pruned
- **WHEN** log retention is configured to keep the latest `N` rotated log files
- **THEN** older rotated log files are removed while newer ones remain available

### Requirement: Logs Must Not Leak Secrets
The system SHALL NOT log sensitive secret material such as remote storage credentials, SMTP passwords, session IDs, enrollment tokens, or encryption private keys.

#### Scenario: Remote target credentials are redacted
- **WHEN** a backup run uses WebDAV credentials stored in encrypted secrets
- **THEN** logs do not contain the WebDAV password or equivalent secret values

### Requirement: Critical Workflows Produce Actionable Logs
The system SHALL emit actionable `INFO`/`WARN`/`ERROR` logs for critical workflows, including run lifecycle, target upload/download, restore/verify, agent connectivity, and notification delivery.

#### Scenario: Run lifecycle is observable
- **WHEN** a run is queued, started, and completed
- **THEN** logs include the run ID and job ID at `INFO` level

#### Scenario: Failures are observable
- **WHEN** a target upload, restore, verify, agent dispatch, or notification send fails
- **THEN** logs include an error summary at `WARN` or `ERROR` level with enough context to diagnose the issue
