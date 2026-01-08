## ADDED Requirements

### Requirement: HTTP Agents Module Is Split Into Focused Submodules
The backend SHALL organize HTTP agent-related handler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Websocket protocol handling is localized
- **WHEN** a developer needs to modify agent websocket message handling
- **THEN** the change primarily occurs in the websocket submodule and does not require edits to admin CRUD handlers

### Requirement: HTTP Agents Refactor Preserves Behavior
The backend SHALL preserve existing HTTP agents behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

