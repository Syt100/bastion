## ADDED Requirements

### Requirement: CI Includes Automated Secret Leak Scanning
The project CI workflow SHALL run an automated secret leak scan to detect likely committed credentials (tokens, API keys, private keys) before changes are merged or released.

#### Scenario: CI fails when a likely secret is detected
- **GIVEN** the repository contains content that matches a secret leak rule
- **WHEN** the CI scripts are executed
- **THEN** the secret scan step fails the run
- **AND** the output is redacted to avoid printing secrets in plaintext logs

