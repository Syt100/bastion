---
## ADDED Requirements

### Requirement: Scheduler Worker Loop Is Split Into Focused Submodules
The backend SHALL organize scheduler worker loop code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Worker orchestration changes are localized
- **WHEN** a developer needs to adjust how queued runs are claimed and processed
- **THEN** changes are primarily localized to the worker loop submodule and do not require edits to dispatch/execute implementations

### Requirement: Scheduler Worker Loop Refactor Preserves Behavior
The backend SHALL preserve existing scheduler worker behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

