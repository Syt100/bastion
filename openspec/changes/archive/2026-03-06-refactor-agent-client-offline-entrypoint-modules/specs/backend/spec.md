---
## ADDED Requirements

### Requirement: Agent Offline Entrypoint Uses Directory Module Layout
The backend SHALL organize agent offline code so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Offline navigation is consistent
- **WHEN** a developer needs to inspect offline scheduling or sync behavior
- **THEN** the entrypoint and its offline submodules are found under the same `offline/` directory

### Requirement: Agent Offline Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing agent offline behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

