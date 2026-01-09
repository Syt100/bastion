---
## ADDED Requirements

### Requirement: Notifications Entrypoint Uses Directory Module Layout
The backend SHALL organize the notifications module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Notifications navigation is consistent
- **WHEN** a developer needs to inspect notification logic
- **THEN** the entrypoint and its notifications submodules are found under the same `notifications/` directory

### Requirement: Notifications Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing notifications behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

