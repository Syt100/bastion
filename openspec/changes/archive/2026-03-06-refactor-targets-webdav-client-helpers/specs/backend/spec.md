---
## ADDED Requirements

### Requirement: WebDAV Client Uses Shared Request Helpers
The backend SHALL reduce duplication in the WebDAV client by extracting shared request helper logic, without changing behavior.

#### Scenario: Auth wiring changes are localized
- **WHEN** a developer needs to adjust how WebDAV requests apply authentication
- **THEN** the change is made in a shared helper rather than duplicated across multiple request methods

### Requirement: WebDAV Client Helper Refactor Preserves Behavior
The backend SHALL preserve existing WebDAV client behavior while refactoring internal helper structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

