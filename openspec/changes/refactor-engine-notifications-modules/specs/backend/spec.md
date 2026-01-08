## ADDED Requirements

### Requirement: Engine Notifications Code Is Split Into Focused Submodules
The backend SHALL organize engine notification code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Template changes are localized
- **WHEN** a developer needs to add or modify template placeholders
- **THEN** the change primarily occurs in the template submodule and does not require edits to sending or worker loop logic

### Requirement: Engine Notifications Refactor Preserves Behavior
The backend SHALL preserve existing notification behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

