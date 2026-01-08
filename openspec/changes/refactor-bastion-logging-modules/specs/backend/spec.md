---
## ADDED Requirements

### Requirement: Logging Code Is Split Into Focused Submodules
The backend SHALL organize logging code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Pruning logic changes are localized
- **WHEN** a developer needs to adjust rotated-log pruning behavior
- **THEN** the change primarily occurs in the pruning submodule and does not require edits to initialization/filtering code

### Requirement: Logging Refactor Preserves Behavior
The backend SHALL preserve existing logging behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

