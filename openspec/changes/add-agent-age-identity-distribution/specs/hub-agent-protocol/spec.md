## ADDED Requirements

### Requirement: Secrets Snapshot Can Include Backup Age Identities
The Hub→Agent secrets snapshot protocol SHALL support distributing backup age identities (`backup_age_identity`) to Agents.

#### Scenario: Agent receives an age identity for restore
- **GIVEN** an encrypted run references key name `K`
- **WHEN** the Hub distributes `backup_age_identity/K` to an Agent and sends an updated secrets snapshot
- **THEN** the Agent persists the key and can decrypt restore payloads that require `K`

