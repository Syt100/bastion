# observability Specification

## Purpose
TBD - created by archiving change refactor-backup-source-target-driver-architecture. Update Purpose after archive.
## Requirements
### Requirement: Backup Events and Metrics MUST Include Driver Dimensions
Backup observability data MUST include source/target driver identity and planner mode dimensions for
both success and failure paths.

#### Scenario: Run events include planner and driver fields
- **WHEN** a run transitions through planning, packaging, upload, and completion/failure stages
- **THEN** emitted events include `source_driver`, `target_driver`, and `plan_mode`
- **AND** fallback events include machine-readable fallback reason fields

### Requirement: Driver Implementations MUST Pass Contract Test Suites
Each source and target driver implementation MUST pass shared contract test suites before release.

#### Scenario: New target driver fails contract tests
- **WHEN** a new target driver implementation violates lifecycle idempotency or cleanup contract
- **THEN** CI contract test suite fails and blocks merge

### Requirement: Cross-Driver Matrix Tests MUST Cover Critical Combinations
CI MUST run matrix tests across source-target-format combinations that are marked as supported.

#### Scenario: Supported combination regresses
- **WHEN** a supported source-target-format combination regresses in planner or runtime behavior
- **THEN** matrix tests fail with the offending combination surfaced in test output

### Requirement: HTTP Requests Have a Correlation ID
The backend SHALL assign a correlation/request ID to each inbound HTTP request and SHALL propagate it to responses to support debugging.

#### Scenario: Request-id is returned to the client
- **WHEN** a client sends an HTTP request without a request-id header
- **THEN** the backend generates a request ID
- **AND** includes it in the response headers

### Requirement: Logs and Spans Include Request Context
When emitting request-scoped logs/spans for HTTP requests, the backend SHALL include the request ID and relevant identifiers (e.g., `job_id`, `run_id`, `operation_id`) without leaking secrets.

#### Scenario: HTTP errors are diagnosable
- **WHEN** an HTTP request fails with a 4xx/5xx error
- **THEN** logs include the request-id and relevant identifiers to correlate client errors with backend logs

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

