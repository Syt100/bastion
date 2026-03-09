## ADDED Requirements

### Requirement: Job Spec MUST Support Driver Envelopes
Backup job specs MUST support a canonical driver envelope for both source and target definitions.
The envelope MUST include `type`, `version`, and driver-specific `config`. Target envelopes MUST
also support explicit `auth_refs`.

#### Scenario: Persist and read canonical V2 driver envelopes
- **WHEN** a user saves a backup job using a source driver and target driver
- **THEN** the stored canonical spec contains `source.type`, `source.version`, `source.config`
- **AND** contains `target.type`, `target.version`, `target.config`, `target.auth_refs`

### Requirement: Job Spec V1 MUST Remain Backward Compatible
The system MUST accept existing V1 job specs and translate them to canonical V2 representation
without changing runtime behavior.

#### Scenario: Execute existing V1 job with V2 runtime
- **WHEN** a legacy V1 filesystem/sqlite/vaultwarden job is loaded
- **THEN** the system translates it to canonical V2 in-memory
- **AND** execution semantics remain equivalent to the pre-migration behavior

### Requirement: Job Spec MUST Keep Credentials Out of Driver Config
Credential material MUST NOT be stored inline in source or target driver configs. Credentials MUST
be referenced via secret references and resolved at runtime in node scope.

#### Scenario: Reject inline target credentials
- **WHEN** a job payload includes direct credential fields inside target config
- **THEN** validation fails with a structured error indicating that secret references are required
