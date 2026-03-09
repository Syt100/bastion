---
## ADDED Requirements

### Requirement: Jobs CRUD Handlers Use Shared Validation Helpers
The backend SHALL reduce duplication in jobs CRUD HTTP handlers by extracting shared validation/normalization helpers, without changing behavior.

#### Scenario: Validation changes are localized
- **WHEN** a developer needs to adjust jobs CRUD validation behavior
- **THEN** changes primarily occur in shared helper functions rather than being duplicated across multiple handlers

### Requirement: Jobs CRUD Helper Refactor Preserves Behavior
The backend SHALL preserve existing jobs CRUD behavior while refactoring internal helper structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

