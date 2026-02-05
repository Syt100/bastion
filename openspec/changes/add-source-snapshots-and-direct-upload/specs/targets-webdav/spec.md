## ADDED Requirements

### Requirement: Raw-Tree Direct Upload To WebDAV Is Atomic (When Enabled)
When raw-tree direct upload to WebDAV is enabled, the system SHALL upload raw-tree payloads without requiring full local staging, while preserving atomic completion semantics.

#### Scenario: No complete marker until all payloads uploaded
- **WHEN** raw-tree direct upload is enabled
- **THEN** the `complete` marker is uploaded last
