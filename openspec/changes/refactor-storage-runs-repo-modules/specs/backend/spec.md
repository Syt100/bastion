---
## ADDED Requirements

### Requirement: Storage Runs Repository Code Is Split Into Focused Submodules
The backend SHALL organize storage runs repository code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Event ingestion changes are localized
- **WHEN** a developer needs to change run event append/query logic
- **THEN** the change primarily occurs in the events submodule and does not require edits to run lifecycle or retention logic

### Requirement: Storage Runs Repository Refactor Preserves Behavior
The backend SHALL preserve existing runs repository behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

