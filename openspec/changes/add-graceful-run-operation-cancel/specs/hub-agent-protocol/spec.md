## ADDED Requirements

### Requirement: Hub Can Dispatch Cancel Requests For Agent Tasks
The Hub↔Agent protocol SHALL support explicit cancel messages for in-flight run tasks and operation tasks.

#### Scenario: Hub requests cancellation of agent run task
- **GIVEN** an agent is executing run task `<run_id>`
- **WHEN** the Hub issues a run cancel request
- **THEN** the agent receives a cancel message for `<run_id>` and starts cooperative stop handling

#### Scenario: Hub requests cancellation of agent operation task
- **GIVEN** an agent is executing operation task `<op_id>`
- **WHEN** the Hub issues an operation cancel request
- **THEN** the agent receives a cancel message for `<op_id>` and starts cooperative stop handling

### Requirement: Agent Reports Cancel-Aware Completion
The Hub↔Agent protocol SHALL allow agent task completion to represent canceled outcomes.

#### Scenario: Agent finishes canceled run
- **GIVEN** an agent run task received a cancel request
- **WHEN** the agent reaches a cancellation checkpoint and exits
- **THEN** task completion to Hub indicates canceled outcome semantics

### Requirement: Cancel Delivery Is Reconnect-Safe
Cancel intent for agent-executed tasks SHALL remain effective across transient Hub↔Agent reconnects.

#### Scenario: Agent reconnects after cancel requested
- **GIVEN** Hub has recorded cancel intent for an in-flight agent task
- **AND** the agent disconnects and reconnects before finishing
- **WHEN** synchronization resumes
- **THEN** the agent still receives/observes cancel intent for that task
