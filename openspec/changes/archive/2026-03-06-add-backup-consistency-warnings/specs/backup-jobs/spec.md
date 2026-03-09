## MODIFIED Requirements

### Requirement: Runs Record Source Consistency Warnings
When source consistency warnings are detected during a backup run, the system SHALL:
- record totals and capped samples in run `summary_json`, and
- emit a warning `run_event` to make the condition visible during and after execution.

#### Scenario: Consistency warnings are persisted in summary
- **WHEN** a filesystem backup detects one or more source changes during packaging
- **THEN** the run summary includes a structured `filesystem.consistency` object with totals and samples

#### Scenario: Consistency warnings are visible in run events
- **WHEN** a backup run detects one or more source changes during packaging
- **THEN** the run emits a `run_event` with `level=warn` and `kind=source_consistency`
- **AND** the event fields include totals and a capped sample list

### Requirement: Consistency Warning Payloads Are Bounded
Consistency warning payloads stored in SQLite (events and summaries) SHALL be bounded to prevent unbounded growth.

#### Scenario: Sample list is capped
- **WHEN** a backup run observes more than the maximum number of changed files
- **THEN** the run summary and event fields include only the capped sample list
- **AND** the totals still reflect the full count

