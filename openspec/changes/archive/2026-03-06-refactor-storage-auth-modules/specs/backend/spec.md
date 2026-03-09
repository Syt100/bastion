---
## ADDED Requirements

### Requirement: Storage Auth Code Is Split Into Focused Submodules
The backend SHALL organize storage auth code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Login throttling changes are localized
- **WHEN** a developer needs to adjust login throttling behavior
- **THEN** the change primarily occurs in the throttle submodule and does not require edits to password hashing or session code

### Requirement: Storage Auth Refactor Preserves Behavior
The backend SHALL preserve existing auth behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

