---
## ADDED Requirements

### Requirement: Scheduler Worker Loop Is Split Into Focused Submodules
The backend SHALL organize scheduler worker loop phases into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Phase logic is easier to modify safely
- **WHEN** a developer adjusts agent dispatch/polling behavior or local execution completion logic
- **THEN** changes are localized to the corresponding submodule rather than requiring edits across one large loop function

### Requirement: Scheduler Worker Loop Refactor Preserves Behavior
The backend SHALL preserve existing scheduling/dispatch/notification behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

