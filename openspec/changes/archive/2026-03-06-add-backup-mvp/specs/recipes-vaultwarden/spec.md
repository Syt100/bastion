## ADDED Requirements

### Requirement: Vaultwarden Recipe (SQLite + Docker/Compose)
The system SHALL provide a Vaultwarden recipe that composes filesystem and SQLite backup primitives to back up Vaultwarden data for Docker/Compose deployments without stopping the service.

#### Scenario: Vaultwarden recipe uses mounted data directory
- **WHEN** the user configures the host path to Vaultwarden's mounted `data/` directory
- **THEN** the recipe backs up the SQLite database snapshot and required data files (attachments and keys)

### Requirement: No Service Downtime
The Vaultwarden recipe SHALL NOT require stopping the Vaultwarden service to perform backups.

#### Scenario: Backup runs while container stays up
- **WHEN** a Vaultwarden backup run is executed
- **THEN** the container remains running and the database snapshot is still consistent

