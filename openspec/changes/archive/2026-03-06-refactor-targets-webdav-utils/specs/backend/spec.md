---
## ADDED Requirements

### Requirement: WebDAV Helpers Are Centralized
The backend SHALL avoid duplicating WebDAV helper logic by centralizing shared helpers, without changing behavior.

#### Scenario: URL redaction logic is consistent
- **WHEN** WebDAV code logs a URL for diagnostics
- **THEN** the same redaction logic is applied across both WebDAV storage and WebDAV client code paths

### Requirement: WebDAV Helper Refactor Preserves Behavior
The backend SHALL preserve existing WebDAV behavior while refactoring internal helper structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

