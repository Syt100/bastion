---
## ADDED Requirements

### Requirement: Agent Managed State Code Is Split Into Focused Submodules
The backend SHALL organize agent managed state code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Task result persistence changes are localized
- **WHEN** a developer needs to adjust how task results are cached on disk
- **THEN** the change primarily occurs in the task-results submodule and does not require edits to managed snapshot encryption logic

### Requirement: Agent Managed State Refactor Preserves Behavior
The backend SHALL preserve existing agent managed behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

