## ADDED Requirements

### Requirement: Hub Forwards Agent Filesystem Browse Requests
When operating in Agent node context, the Hub SHALL forward filesystem list requests to the Agent over the existing websocket control channel and SHALL return the Agent’s response to the Web UI.

#### Scenario: Agent is offline
- **WHEN** the user requests a filesystem listing for an Agent node that is not connected
- **THEN** the API returns a clear error indicating the Agent is offline

