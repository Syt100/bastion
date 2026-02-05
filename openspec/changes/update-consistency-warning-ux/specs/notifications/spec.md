## MODIFIED Requirements

### Requirement: Notifications Mention Consistency Warnings
When a run completes and has source consistency warnings, notifications SHALL explicitly mention that the source changed during backup and include the warning count.

#### Scenario: Notification includes consistency warning line
- **WHEN** a run completes with `consistency_changed_total > 0`
- **THEN** the sent notification includes a line indicating the source changed during backup and includes the count

