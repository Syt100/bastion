## ADDED Requirements

### Requirement: HTTP Notifications Module Is Split Into Focused Submodules
The backend SHALL organize HTTP notification-related handler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Queue handling is localized
- **WHEN** a developer needs to modify notification queue operations (list/cancel/retry)
- **THEN** the change primarily occurs in the queue submodule and does not require edits to settings or destination management

### Requirement: HTTP Notifications Refactor Preserves Behavior
The backend SHALL preserve existing HTTP notifications behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

