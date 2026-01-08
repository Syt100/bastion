## ADDED Requirements

### Requirement: Filesystem Backup Module Is Split Into Focused Submodules
The backend SHALL organize filesystem backup implementation code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Filesystem backup code navigation is localized
- **WHEN** a developer needs to change entries index serialization
- **THEN** the change primarily occurs in the entries index module and does not require edits to tar writing logic

### Requirement: Filesystem Backup Refactor Preserves Behavior
The backend SHALL preserve existing filesystem backup behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

