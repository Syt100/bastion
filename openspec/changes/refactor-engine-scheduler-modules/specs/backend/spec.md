## ADDED Requirements

### Requirement: Scheduler Module Is Split Into Focused Submodules
The backend SHALL organize scheduler implementation code into focused Rust modules to improve maintainability, without changing scheduler behavior.

#### Scenario: Scheduler code navigation is localized
- **WHEN** a developer needs to change cron normalization behavior
- **THEN** the change primarily occurs in the scheduler cron module and does not require edits to queue/orchestration logic

### Requirement: Scheduler Refactor Preserves Behavior
The backend SHALL preserve existing scheduler behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing scheduler tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

