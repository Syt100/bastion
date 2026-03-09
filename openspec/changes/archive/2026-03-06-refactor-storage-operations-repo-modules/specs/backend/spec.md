---
## ADDED Requirements

### Requirement: Operations Repo Code Is Split Into Focused Submodules
The backend SHALL organize operations repository code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Type changes are isolated from DB queries
- **WHEN** a developer needs to add a new `OperationKind`
- **THEN** the primary change occurs in the operations types submodule and DB query logic remains localized in the repo submodule

### Requirement: Operations Repo Refactor Preserves Behavior
The backend SHALL preserve existing operations repository behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

