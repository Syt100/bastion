---
## ADDED Requirements

### Requirement: Age Identity Distribution Is Auditable and Does Not Leak Secrets
When distributing backup age identities to Agents, the backend SHALL record audit-friendly events and SHALL NOT log secret payload values.

#### Scenario: Logs contain key names but not key values
- **WHEN** an age identity secret `backup_age_identity/K` is distributed
- **THEN** logs/events may reference `K` and the target Agent
- **AND** logs/events MUST NOT include the identity private key contents

