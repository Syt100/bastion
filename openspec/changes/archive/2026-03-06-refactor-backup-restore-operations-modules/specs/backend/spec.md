---
## ADDED Requirements

### Requirement: Restore Operations Code Is Split Into Focused Submodules
The backend SHALL organize restore operations code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Restore and verify logic are separated
- **WHEN** a developer needs to adjust restore or verify behavior
- **THEN** changes are localized to the corresponding restore/verify submodule and do not require edits to the other flow

### Requirement: Restore Operations Refactor Preserves Behavior
The backend SHALL preserve existing restore operations behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

