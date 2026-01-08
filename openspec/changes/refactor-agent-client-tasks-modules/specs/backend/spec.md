---
## ADDED Requirements

### Requirement: Agent Tasks Code Is Split Into Focused Submodules
The backend SHALL organize agent task handling code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Per-spec backup logic changes are localized
- **WHEN** a developer needs to adjust filesystem backup behavior
- **THEN** the change primarily occurs in the filesystem task submodule and does not require edits to sqlite or vaultwarden handlers

### Requirement: Agent Tasks Refactor Preserves Behavior
The backend SHALL preserve existing agent task handling behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

