---
## ADDED Requirements

### Requirement: Storage Secrets Code Is Split Into Focused Submodules
The backend SHALL organize storage secrets (keyring/crypto/keypack) code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Keypack changes are localized
- **WHEN** a developer needs to change keypack export/import handling
- **THEN** the change primarily occurs in the keypack submodule and does not require edits to encryption/decryption logic

### Requirement: Storage Secrets Refactor Preserves Behavior
The backend SHALL preserve existing secrets behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

