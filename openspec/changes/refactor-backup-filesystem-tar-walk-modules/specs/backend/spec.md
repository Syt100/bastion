---
## ADDED Requirements

### Requirement: Filesystem Tar Walking Code Is Split Into Focused Submodules
The backend SHALL organize filesystem tar walking code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Legacy root logic is localized
- **WHEN** a developer needs to adjust legacy `filesystem.source.root` handling
- **THEN** the change primarily occurs in the legacy-root submodule and does not require edits to selected-path walking logic

### Requirement: Filesystem Tar Walk Refactor Preserves Behavior
The backend SHALL preserve existing filesystem tar walking behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

