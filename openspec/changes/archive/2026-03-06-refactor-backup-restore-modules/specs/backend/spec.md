## ADDED Requirements

### Requirement: Restore Module Is Split Into Focused Submodules
The backend SHALL organize backup restore implementation code into focused Rust modules (entries index listing, access resolution, operation orchestration, unpacking, and verification) to improve maintainability.

#### Scenario: Restore code navigation is localized
- **WHEN** a developer needs to change restore entries listing behavior
- **THEN** the change primarily occurs in the entries index module and does not require edits to unpacking logic

### Requirement: Restore Refactor Preserves Behavior
The backend SHALL preserve existing restore/verify behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

