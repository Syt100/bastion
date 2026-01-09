---
## ADDED Requirements

### Requirement: Restore Entries Index Code Is Split Into Focused Submodules
The backend SHALL organize restore entries index code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Fetching and listing logic are separated
- **WHEN** a developer needs to adjust entries index caching or download behavior
- **THEN** changes are localized to the fetch/cache submodule and do not require edits to listing/filtering code

### Requirement: Restore Entries Index Refactor Preserves Behavior
The backend SHALL preserve existing restore entries index behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

