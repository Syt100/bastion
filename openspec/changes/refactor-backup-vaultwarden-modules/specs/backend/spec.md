---
## ADDED Requirements

### Requirement: Vaultwarden Backup Code Is Split Into Focused Submodules
The backend SHALL organize vaultwarden backup code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Hashing logic is localized
- **WHEN** a developer needs to adjust hashing behavior or file IO helpers
- **THEN** the change primarily occurs in the hashing/IO submodule and does not require edits to tar walking logic

### Requirement: Vaultwarden Backup Refactor Preserves Behavior
The backend SHALL preserve existing vaultwarden backup behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

