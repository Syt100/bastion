## ADDED Requirements

### Requirement: Crate Split Supports Incremental Development
The codebase SHALL maintain a modular crate structure that enables faster incremental development by isolating high-churn code from heavy/low-churn code.

#### Scenario: Routine changes stay localized
- **WHEN** a developer changes an HTTP handler or WebSocket message mapping
- **THEN** the change is localized to the HTTP crate (and its direct dependencies) rather than forcing unrelated backup/target implementation changes

