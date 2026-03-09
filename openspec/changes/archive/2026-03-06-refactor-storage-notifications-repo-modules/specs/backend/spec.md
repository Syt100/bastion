---
## ADDED Requirements

### Requirement: Storage Notifications Repository Code Is Split Into Focused Submodules
The backend SHALL organize storage notification repository code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Enqueue changes are localized
- **WHEN** a developer needs to change enqueue selection or insertion logic
- **THEN** the change primarily occurs in the enqueue submodule and does not require edits to claiming or queue query logic

### Requirement: Storage Notifications Repository Refactor Preserves Behavior
The backend SHALL preserve existing notification repository behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

