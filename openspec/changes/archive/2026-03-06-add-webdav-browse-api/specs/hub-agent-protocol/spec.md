## ADDED Requirements

### Requirement: Agent Supports WebDAV Directory Listing for Node-Scoped API
The Hub↔Agent protocol SHALL support a node-scoped WebDAV list request so that the Hub can browse WebDAV paths from the Agent’s network vantage point.

#### Scenario: Hub forwards a WebDAV list request to an Agent
- **GIVEN** an Agent is connected
- **WHEN** the Hub forwards a WebDAV list request for `node_id=<agent_id>`
- **THEN** the Agent performs the PROPFIND and returns entries and paging cursor

