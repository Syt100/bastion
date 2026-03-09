## ADDED Requirements

### Requirement: Agent Offline Module Is Split Into Focused Submodules
The backend SHALL organize agent offline scheduling/sync code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Cron matching logic is localized
- **WHEN** a developer needs to modify cron parsing/normalization
- **THEN** the change primarily occurs in the cron submodule and does not require edits to offline run persistence

### Requirement: Agent Offline Refactor Preserves Behavior
The backend SHALL preserve existing agent offline behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

