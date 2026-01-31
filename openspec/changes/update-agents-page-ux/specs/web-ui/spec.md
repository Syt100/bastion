## ADDED Requirements

### Requirement: Agents Page Quick Links to Node Context
The Agents page SHALL provide quick navigation to common node-scoped pages for a given agent.

#### Scenario: Jump to agent Jobs
- **GIVEN** an agent is listed on the Agents page
- **WHEN** the user clicks “Jobs”
- **THEN** the UI navigates to `/n/:agentId/jobs`

#### Scenario: Jump to agent Storage
- **GIVEN** an agent is listed on the Agents page
- **WHEN** the user clicks “Storage”
- **THEN** the UI navigates to `/n/:agentId/settings/storage`

### Requirement: Enrollment Token Provides Command Template
The enrollment token modal SHALL display a copyable CLI command template.

#### Scenario: Token modal shows enroll command
- **GIVEN** an enrollment token is created
- **THEN** the UI shows a copyable command template containing:
  - the Hub URL
  - the enroll token value
  - placeholders for agent name

