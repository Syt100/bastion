## ADDED Requirements

### Requirement: Scheduler Worker Is Split Into Focused Submodules
The backend SHALL organize scheduler worker code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Dispatch logic is localized
- **WHEN** a developer needs to modify agent dispatch behavior
- **THEN** the change primarily occurs in the dispatch submodule and does not require edits to local run execution or target storage code

### Requirement: Scheduler Worker Refactor Preserves Behavior
The backend SHALL preserve existing scheduler worker behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

