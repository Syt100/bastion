## ADDED Requirements

### Requirement: Hub Can Distribute Backup Age Identities to Selected Agents
The Hub SHALL support copying a backup age identity secret from Hub scope to a selected Agent scope to enable Agent-executed encrypted restores.

#### Scenario: Hub distributes a missing key on-demand
- **GIVEN** a restore is requested to execute on Agent `<agent_id>`
- **AND** the run references age key name `K`
- **AND** Agent `<agent_id>` does not have `backup_age_identity/K`
- **WHEN** the restore is started
- **THEN** the Hub copies the key to Agent scope and refreshes secrets delivery before the restore begins

