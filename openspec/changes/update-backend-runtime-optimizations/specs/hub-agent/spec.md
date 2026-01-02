## ADDED Requirements

### Requirement: Agent Reconnect Uses Jittered Backoff
When reconnecting to the Hub, Agents SHALL use backoff with jitter to avoid synchronized reconnect storms.

#### Scenario: Reconnect includes jitter
- **WHEN** an Agent reconnects repeatedly due to network instability
- **THEN** the retry delay includes jitter so multiple agents do not reconnect at the exact same cadence

### Requirement: Agent Heartbeat Has a Pong Timeout
Agents SHALL use heartbeat ping/pong and SHALL reconnect when a pong is not observed within a configured timeout window.

#### Scenario: Missed pong triggers reconnect
- **WHEN** the Agent does not observe a pong within the timeout window
- **THEN** it reconnects to the Hub

### Requirement: Task ACK and Retry Boundaries Reduce Duplicate Work
The Hub/Agent protocol SHALL define clear ACK and retry boundaries so that reconnect-driven re-delivery does not cause excessive duplicate work.

#### Scenario: Duplicate task deliveries are handled safely
- **WHEN** a task is delivered more than once due to reconnect
- **THEN** the Agent handles the duplicate delivery without executing the task multiple times when avoidable
