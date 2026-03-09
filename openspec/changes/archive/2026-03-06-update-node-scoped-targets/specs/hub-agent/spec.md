## ADDED Requirements

### Requirement: Hub Stores Node-Scoped Configuration
The Hub SHALL store node-scoped target/credential configuration for the Hub node and for Agent nodes.

#### Scenario: Agent has isolated targets
- **WHEN** two Agents are enrolled (A and B)
- **THEN** targets created for Agent A are not visible/usable for Agent B

### Requirement: Managed Agent Configuration Is Read-Only Locally
When an Agent is enrolled and managed by a Hub, any local configuration UI (if enabled) SHALL be read-only for configuration changes and MUST direct users to manage configuration in the Hub UI.

#### Scenario: Agent config UI is read-only
- **WHEN** a user opens the Agent local UI while it is enrolled
- **THEN** configuration editors are disabled and the UI indicates management is done in the Hub

