---
## ADDED Requirements

### Requirement: Jobs Repo Code Is Split Into Focused Submodules
The backend SHALL organize jobs repository code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Overlap policy changes are isolated from DB queries
- **WHEN** a developer needs to update `OverlapPolicy` parsing/serialization
- **THEN** the change primarily occurs in the jobs types submodule and DB query logic remains localized in the repo submodule

### Requirement: Jobs Repo Refactor Preserves Behavior
The backend SHALL preserve existing jobs repository behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

