---
## ADDED Requirements

### Requirement: WebDAV Secret Handlers Use Shared Helpers
The backend SHALL reduce duplication in WebDAV secret HTTP handlers by extracting shared validation and persistence helpers, without changing behavior.

#### Scenario: Changes are localized
- **WHEN** a developer adjusts WebDAV secret validation or persistence behavior
- **THEN** changes primarily occur in shared helper functions rather than being duplicated across hub-level and node-level handlers

### Requirement: WebDAV Secret Helper Refactor Preserves Behavior
The backend SHALL preserve existing WebDAV secret CRUD behavior while refactoring internal helper structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

