---
## ADDED Requirements

### Requirement: Scheduler Entrypoint Uses Directory Module Layout
The backend SHALL organize the scheduler module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Scheduler navigation is consistent
- **WHEN** a developer needs to inspect scheduling logic
- **THEN** the scheduler entrypoint and its submodules are found under the same `scheduler/` directory

### Requirement: Scheduler Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing scheduler behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

