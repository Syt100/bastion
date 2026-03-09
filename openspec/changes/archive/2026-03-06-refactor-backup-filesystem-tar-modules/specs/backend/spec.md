## ADDED Requirements

### Requirement: Filesystem Tar Writer Is Split Into Focused Submodules
The backend SHALL organize filesystem tar writing code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Hardlink handling is localized
- **WHEN** a developer needs to modify hardlink behavior
- **THEN** the change primarily occurs in the entry-writing submodule and does not require edits to encryption/part orchestration

### Requirement: Filesystem Tar Refactor Preserves Behavior
The backend SHALL preserve existing filesystem tar output behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

