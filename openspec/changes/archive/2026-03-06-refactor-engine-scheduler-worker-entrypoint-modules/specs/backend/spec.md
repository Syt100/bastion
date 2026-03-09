---
## ADDED Requirements

### Requirement: Scheduler Worker Entrypoint Uses Directory Module Layout
The backend SHALL organize the scheduler worker module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Worker navigation is consistent
- **WHEN** a developer needs to inspect how runs are processed
- **THEN** the worker entrypoint and its submodules are found under the same `worker/` directory

### Requirement: Scheduler Worker Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing scheduler worker behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

