---
## ADDED Requirements

### Requirement: Backup Filesystem Entrypoint Uses Directory Module Layout
The backend SHALL organize the backup filesystem module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Filesystem backup navigation is consistent
- **WHEN** a developer needs to inspect filesystem backup building logic
- **THEN** the entrypoint and its related modules are found under the same `filesystem/` directory

### Requirement: Backup Filesystem Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing filesystem backup behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

