## ADDED Requirements

### Requirement: Global and Channel Notification Switches
The system SHALL provide a global notifications enable switch and per-channel enable switches (WeCom group bot, Email/SMTP).

When the global switch is turned off, the system SHALL stop enqueueing new notifications and SHALL cancel queued notifications.
When a channel switch is turned off, the system SHALL stop enqueueing new notifications for that channel and SHALL cancel queued notifications for that channel.

#### Scenario: Global notifications are disabled
- **WHEN** the user turns off global notifications
- **THEN** no new notifications are enqueued for any channel
- **AND** queued notifications are marked as canceled (in-scope)

#### Scenario: A channel is disabled
- **WHEN** the user turns off the Email channel
- **THEN** no new Email notifications are enqueued
- **AND** queued Email notifications are marked as canceled (in-scope)

### Requirement: Notification Destinations Can Be Enabled/Disabled
The system SHALL support enabling and disabling individual notification destinations.

Disabled destinations SHALL NOT receive notifications and MAY still be selectable in per-job configuration, but the UI MUST show inline warnings when disabled destinations are selected.

#### Scenario: Disabled destination does not receive notifications
- **WHEN** a destination is disabled
- **THEN** notifications to that destination are not enqueued and not sent

### Requirement: Destination Deletion Cancels Queue Entries
When a notification destination is deleted, the system SHALL cancel queued notifications targeting that destination and SHALL stop retrying them.

Historical notifications that previously failed SHALL remain in `failed` state, but the UI SHOULD indicate the destination has been deleted.

#### Scenario: Delete destination cancels queued notifications
- **WHEN** the user deletes a WeCom destination
- **THEN** queued notifications targeting that destination are canceled and not retried

### Requirement: Notification Templates (Global Only)
The system SHALL support global notification templates for:
- WeCom markdown content
- Email subject
- Email body

Templates SHALL support basic placeholder substitution using run/job fields.

#### Scenario: Template renders run details
- **WHEN** a run completes
- **THEN** the sent notification content is generated from the configured template with placeholders replaced

### Requirement: Notification Queue/Records with Pagination and Actions
The system SHALL provide a paginated notification queue/records view with filters (at least by status and channel) and SHALL surface delivery state and last error (if any).

The system SHALL provide queue actions:
- Retry now: schedule immediate retry and reset attempts to 0
- Cancel: cancel a queued notification so it is not sent or retried

#### Scenario: Retry now schedules immediate send
- **WHEN** a user clicks Retry now for a failed notification
- **THEN** the notification is scheduled immediately
- **AND** attempts is reset to 0

#### Scenario: Cancel prevents send
- **WHEN** a user cancels a queued notification
- **THEN** the notification is marked canceled and is not sent

### Requirement: Synchronous Test Notification
The system SHALL allow sending a test notification to a destination and SHALL return the result synchronously (not queued).

#### Scenario: Test send returns immediate result
- **WHEN** a user sends a test notification to an SMTP destination
- **THEN** the API returns success/failure immediately with an actionable message

