## ADDED Requirements

### Requirement: Documentation Covers Safe Upgrade and Rollback
Project documentation SHALL include an Upgrade & rollback guide for common deployment methods:
- Docker (volume-backed)
- Linux packages (`.deb`/`.rpm` + systemd)
- Windows MSI (service install)
- Portable tar/zip installs

The guide MUST explain:
- how to identify the current version
- how to back up the data directory (SQLite + secrets) before upgrading
- that schema migrations may be irreversible and rollback may require restoring the backup

#### Scenario: Operator follows upgrade guide
- **WHEN** an operator wants to upgrade Bastion
- **THEN** they can follow documented steps to back up data, upgrade, verify, and roll back safely if needed
