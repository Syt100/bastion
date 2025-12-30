## ADDED Requirements

### Requirement: Email Notifications
The system SHALL support email notifications on backup completion (success or failure) and SHALL support retry with backoff.

#### Scenario: Failure email is retried
- **WHEN** sending a failure email fails temporarily
- **THEN** the system retries sending according to configured policy

### Requirement: WeCom Group Bot Notifications
The system SHALL support WeCom group bot webhook notifications on backup completion (success or failure) and SHALL support retry with backoff.

#### Scenario: Success message is delivered
- **WHEN** a run succeeds
- **THEN** a success notification is sent to the configured WeCom group bot

