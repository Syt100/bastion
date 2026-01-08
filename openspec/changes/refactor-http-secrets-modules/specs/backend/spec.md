## ADDED Requirements

### Requirement: HTTP Secrets Module Is Split Into Focused Submodules
The backend SHALL organize HTTP secret-related handler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Secret types are localized
- **WHEN** a developer needs to modify WebDAV secret handling
- **THEN** the change primarily occurs in the WebDAV submodule and does not require edits to SMTP or WeCom secret handlers

### Requirement: HTTP Secrets Refactor Preserves Behavior
The backend SHALL preserve existing HTTP secrets behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

