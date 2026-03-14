## ADDED Requirements

### Requirement: Integrations Groups External Dependency Workflows
The Web UI SHALL provide an `Integrations` surface that groups storage, notifications, and distribution workflows.

#### Scenario: Integrations index exposes grouped entry points
- **WHEN** an operator opens the Integrations surface
- **THEN** the page SHALL expose entry points for storage, notifications, and distribution management
- **AND** those workflows SHALL be grouped as integration concerns rather than generic settings

#### Scenario: Old settings routes can be normalized into Integrations during migration
- **GIVEN** the operator opens an old settings route for notifications or node-scoped storage that is still covered by a temporary client-side alias
- **WHEN** the route is resolved during migration
- **THEN** the app SHALL normalize to the canonical Integrations route family
- **AND** any applicable scope context SHALL be preserved through query state rather than node-prefixed path identity

### Requirement: Storage Management Shows Usage And Validation Context
Storage-related integration pages SHALL show where credentials or targets are used and whether they appear healthy.

#### Scenario: Storage credential view shows references and validation summary
- **WHEN** an operator inspects a storage credential or storage integration item
- **THEN** the page SHALL show at least its identity, where it is used, and its latest validation or health signal when available

#### Scenario: Node-scoped storage context uses query scope
- **WHEN** an operator narrows storage management to hub or one agent
- **THEN** the page SHALL use explicit query scope or equivalent visible context instead of a node-prefixed route family
- **AND** the storage page SHALL remain part of the Integrations information architecture

### Requirement: Notification Management Shows Delivery Health
Notification-related integration pages SHALL show delivery health and queue context rather than only static configuration forms.

#### Scenario: Notification integrations show failed queue context
- **GIVEN** notification delivery has recent failures
- **WHEN** the operator opens notification management
- **THEN** the page SHALL surface failure or queue-health context alongside channel/destination/template configuration

### Requirement: Distribution Management Shows Coverage And Drift
Distribution-related integration pages SHALL show how configuration or secret distribution is applied across nodes.

#### Scenario: Distribution page shows missing or failed application state
- **WHEN** distributed credentials or related integration state are not fully applied across nodes
- **THEN** the distribution management page SHALL show coverage or drift state per relevant scope
- **AND** it SHALL provide direct follow-up actions or navigation for remediation

### Requirement: Integrations Uses Aggregated Integration View Models
The system SHALL provide aggregated integration-oriented summaries so the UI can render usage, validation, and coverage context without stitching together many unrelated endpoints.

#### Scenario: Integrations index loads grouped summaries
- **WHEN** the UI requests integration summary data
- **THEN** the response SHALL provide grouped summaries for storage, notifications, and distribution workflows
- **AND** each summary SHALL be individually renderable even if another integration domain has no data

#### Scenario: Partial integration failure degrades one domain without blanking the others
- **GIVEN** one integration domain cannot produce full summary data
- **WHEN** the Integrations index response is returned
- **THEN** the affected domain SHALL carry explicit degraded state
- **AND** unaffected domains SHALL remain renderable in the same response
