## ADDED Requirements

### Requirement: Enrollment Tokens
The Hub SHALL provide enrollment tokens with a configurable expiration time and SHALL allow limiting token usage.

#### Scenario: Expired token is rejected
- **WHEN** an Agent attempts to enroll using an expired token
- **THEN** enrollment fails

### Requirement: Agent Registration and Long-Lived Key
The Hub SHALL exchange a valid enrollment token for an `agent_id` and long-lived `agent_key`, and the Hub SHALL store only a verifier (hash) of the `agent_key`.

#### Scenario: Agent is issued a key
- **WHEN** an Agent enrolls with a valid token
- **THEN** it receives an `agent_id` and `agent_key` and can connect without the enrollment token

### Requirement: Agent-Initiated WebSocket Connection
Agents SHALL initiate a WebSocket connection to the Hub and SHALL send a `hello` message containing version and capability information.

#### Scenario: Hub records agent capabilities
- **WHEN** an Agent connects and sends `hello`
- **THEN** the Hub records the Agent as online with its declared capabilities

### Requirement: Task Dispatch and Acknowledgement
The Hub SHALL dispatch tasks to Agents and Agents SHALL acknowledge receipt, enabling reconnect-safe delivery.

#### Scenario: Task is acknowledged
- **WHEN** the Hub sends a task with a `task_id`
- **THEN** the Agent sends an acknowledgement for that `task_id`

### Requirement: Agent Key Revocation
The Hub SHALL support revoking an Agent key and SHALL reject subsequent connections using the revoked key.

#### Scenario: Revoked Agent cannot connect
- **WHEN** an Agent key is revoked
- **THEN** the Agent's next connection attempt is rejected

