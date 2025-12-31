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

### Requirement: WeCom Webhook Secrets Storage
The system SHALL store WeCom group bot webhook URLs as encrypted secrets and SHALL allow the user to view and update them via the Web UI.

#### Scenario: Webhook is stored securely
- **WHEN** a user configures a WeCom webhook URL
- **THEN** it is stored encrypted at rest (protected by `data/master.key`)

### Requirement: Notification Dedupe Per Run
The system SHALL deduplicate notifications per run per destination to avoid duplicate alerts on retries.

#### Scenario: Duplicate notifications are not sent
- **WHEN** a run completion notification is enqueued multiple times for the same destination
- **THEN** only one notification is sent
