---
## MODIFIED Requirements

### Requirement: Cleanup Tasks Are Observable and Operable
The backend SHALL expose authenticated APIs to list cleanup tasks, view attempt history, and perform operator actions (retry/ignore/unignore).

#### Scenario: Operator can list tasks filtered by multiple statuses
- **GIVEN** the system has cleanup tasks in multiple statuses
- **WHEN** a user lists cleanup tasks with multiple `status[]` query params
- **THEN** the response contains tasks whose status matches any of the provided statuses

#### Scenario: Operator can list tasks filtered by multiple target types
- **GIVEN** the system has cleanup tasks for multiple target types
- **WHEN** a user lists cleanup tasks with multiple `target_type[]` query params
- **THEN** the response contains tasks whose target type matches any of the provided target types

### Requirement: Notifications Queue Is Queryable
The backend SHALL expose authenticated APIs to list notification queue items and perform operator actions (retry/cancel).

#### Scenario: Operator can list queue items filtered by multiple statuses
- **GIVEN** the system has notification queue items in multiple statuses
- **WHEN** a user lists the queue with multiple `status[]` query params
- **THEN** the response contains items whose status matches any of the provided statuses

#### Scenario: Operator can list queue items filtered by multiple channels
- **GIVEN** the system has notification queue items for multiple channels
- **WHEN** a user lists the queue with multiple `channel[]` query params
- **THEN** the response contains items whose channel matches any of the provided channels
