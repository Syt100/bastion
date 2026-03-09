## ADDED Requirements

### Requirement: Test Credentials MUST Avoid Hard-Coded Secret Literals
Backend test code MUST avoid static credential/password literals that are interpreted as hard-coded cryptographic values by security scanners.

#### Scenario: Auth test setup needs a password
- **WHEN** a test creates a user/session requiring a password
- **THEN** it uses a runtime-generated passphrase value instead of a hard-coded literal

#### Scenario: Keypack tests need import/export password
- **WHEN** tests call keypack export/import helpers with a password
- **THEN** they use generated passphrase values, including wrong-password branches

### Requirement: Secret-Bearing Tests MUST Avoid Cleartext Value Emission
Backend tests that handle decrypted or secret-bearing values MUST not format those values into logs/panic/debug output.

#### Scenario: Secret equality assertion fails
- **WHEN** a secret-bearing assertion fails
- **THEN** the failure message does not include raw secret bytes/string values

#### Scenario: Unexpected enum variant in secret flow
- **WHEN** tests guard secret-bearing enum branches
- **THEN** panic/error messages avoid `{:?}` dumps that could include sensitive fields
