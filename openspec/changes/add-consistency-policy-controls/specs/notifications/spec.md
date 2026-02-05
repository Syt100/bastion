## MODIFIED Requirements

### Requirement: Notifications Mention Consistency Policy Failures
When a run fails due to consistency policy enforcement, notifications SHALL indicate that the source changed during backup and that the run was failed by policy.

#### Scenario: Notification includes policy failure explanation
- **WHEN** a run fails with `error_code="source_consistency"`
- **THEN** the notification includes an explicit “failed due to source changes during backup” message

