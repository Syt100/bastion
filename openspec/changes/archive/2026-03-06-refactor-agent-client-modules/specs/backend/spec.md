## ADDED Requirements

### Requirement: Agent Client Is Split Into Focused Submodules
The backend SHALL organize agent client code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Task handling is localized
- **WHEN** a developer needs to modify backup task handling
- **THEN** the change primarily occurs in the task-handling submodule and does not require edits to identity or websocket connection logic

### Requirement: Agent Client Refactor Preserves Behavior
The backend SHALL preserve existing agent client behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

