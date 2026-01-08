## ADDED Requirements

### Requirement: HTTP Jobs Module Is Split Into Focused Submodules
The backend SHALL organize HTTP job-related handler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Validation logic is localized
- **WHEN** a developer needs to modify job spec validation rules
- **THEN** the change primarily occurs in the validation submodule and does not require edits to websocket or CRUD handlers

### Requirement: HTTP Jobs Refactor Preserves Behavior
The backend SHALL preserve existing HTTP jobs behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

