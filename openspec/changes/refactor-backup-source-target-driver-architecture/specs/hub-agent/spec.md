## ADDED Requirements

### Requirement: Hub and Agent MUST Exchange Driver Capability Metadata
Hub-Agent protocol payloads MUST include driver identifiers and capability metadata needed for
planning and compatibility checks.

#### Scenario: Agent advertises installed driver capabilities
- **WHEN** an agent connects or refreshes config snapshot state
- **THEN** agent exposes source/target driver identifiers and capability metadata to Hub

### Requirement: Hub MUST Enforce Capability-Aware Dispatch
Hub MUST dispatch runs to an agent only when the agent can satisfy required source/target driver
and protocol capabilities.

#### Scenario: Dispatch blocked by missing target driver
- **WHEN** a job requires a target driver that the selected agent does not provide
- **THEN** dispatch is rejected with a structured capability mismatch error

### Requirement: Protocol Evolution MUST Support Mixed-Version Rollouts
The protocol MUST support mixed Hub-Agent versions during migration from enum-based specs to
driver-envelope specs.

#### Scenario: New Hub with old Agent
- **WHEN** Hub supports driver-envelope metadata and agent only supports legacy fields
- **THEN** Hub uses compatibility mode to preserve run execution without data loss
