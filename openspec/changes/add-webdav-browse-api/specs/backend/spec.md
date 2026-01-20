---
## ADDED Requirements

### Requirement: WebDAV Browsing Uses Stored Credentials and Does Not Leak Secrets
When listing WebDAV directories for browsing, the backend SHALL use stored WebDAV credentials and SHALL NOT log secret payload values.

#### Scenario: Logs redact credentials
- **WHEN** the system performs a WebDAV list operation
- **THEN** logs may include a redacted URL but MUST NOT include the WebDAV password

