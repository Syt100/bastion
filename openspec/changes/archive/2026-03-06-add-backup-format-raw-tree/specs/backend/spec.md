---
## ADDED Requirements

### Requirement: Raw-Tree Format Disables Encryption
When a job is configured to use the `raw_tree_v1` artifact format, the backend SHALL reject payload encryption settings that require tar-based packaging.

#### Scenario: Raw-tree with encryption is rejected
- **WHEN** a user configures a job with artifact format `raw_tree_v1` and enables age encryption
- **THEN** the backend rejects the configuration with a clear validation error

