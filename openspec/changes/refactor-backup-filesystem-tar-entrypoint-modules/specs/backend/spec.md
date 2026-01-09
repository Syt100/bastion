---
## ADDED Requirements

### Requirement: Filesystem Tar Entrypoint Uses Directory Module Layout
The backend SHALL organize the filesystem tar module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Tar code navigation is consistent
- **WHEN** a developer needs to inspect tar packaging logic
- **THEN** the entrypoint and its tar-related submodules are found under the same `tar/` directory

### Requirement: Filesystem Tar Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing filesystem tar behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

