## MODIFIED Requirements

### Requirement: Built-In Scheduler
The system SHALL provide a built-in scheduler to execute jobs based on cron expressions.

For jobs assigned to an Agent node, the Agent SHOULD continue scheduled execution using its cached schedule when the Hub is unreachable.

#### Scenario: Agent runs scheduled job while Hub is unreachable
- **WHEN** the Hub is unreachable for an enrolled Agent and a cached job schedule matches the current time
- **THEN** the Agent starts a run locally
- **AND** the run is synced back to the Hub when connectivity returns

