## ADDED Requirements

### Requirement: WebDAV Targets Are Node-Scoped
WebDAV targets and their referenced credentials SHALL be node-scoped.

#### Scenario: WebDAV credential belongs to a node
- **WHEN** a WebDAV target is created for a node
- **THEN** its credential material is stored and referenced within that node scope only

