---
## ADDED Requirements

### Requirement: HTTP Submodule Entrypoints Use Directory Module Layout
The backend SHALL organize HTTP submodules so that each entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: HTTP handler navigation is consistent
- **WHEN** a developer needs to inspect HTTP request handling for agents/jobs/notifications/secrets
- **THEN** the entrypoint and related submodules are found under the same subdirectory

### Requirement: HTTP Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing HTTP behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

