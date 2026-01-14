## ADDED Requirements

### Requirement: Bulk Distribution of WebDAV Secrets Uses Label Selectors
The backend SHALL support bulk distribution of a WebDAV secret to multiple target nodes selected by explicit node ids or label selectors.

#### Scenario: Selector resolves target nodes
- **GIVEN** multiple agents exist
- **WHEN** an operator starts a bulk WebDAV distribution using a label selector
- **THEN** the backend MUST target the resolved node set

### Requirement: Default Behavior Skips Existing Secrets
If the target node already has a WebDAV secret with the destination name, the system SHALL skip that node by default.

#### Scenario: Existing secret is skipped by default
- **GIVEN** a target node already has a WebDAV secret named `primary`
- **WHEN** the operator runs a bulk distribution of secret `primary` with overwrite disabled
- **THEN** that node’s bulk item MUST be marked as skipped

### Requirement: Overwrite Is Explicit and Updates the Secret
The backend SHALL support an explicit overwrite option that replaces the existing secret on a target node.

#### Scenario: Overwrite replaces the secret
- **GIVEN** a target node already has a WebDAV secret named `primary`
- **WHEN** the operator runs the bulk distribution with overwrite enabled
- **THEN** the target node MUST receive the updated secret payload

### Requirement: Distribution Re-encrypts Per Node
When copying secrets between nodes, the backend SHALL re-encrypt the secret for the target node scope.

#### Scenario: Copied secret remains readable on target
- **WHEN** a secret is distributed to a target node
- **THEN** subsequent reads of that node’s secret MUST succeed

### Requirement: Preview Before Execution
The backend SHALL support a preview capability that allows the UI to show, per node, whether the operation would skip, update, or fail before execution.

#### Scenario: Preview shows skip vs update
- **GIVEN** a mix of nodes with and without the destination secret
- **WHEN** the operator requests a bulk distribution preview
- **THEN** the preview MUST indicate which nodes will be skipped and which will be updated

### Requirement: Distribution Triggers Config Refresh or Pending Delivery
After a successful distribution, the system SHALL ensure affected nodes receive updated configuration:
- If online, attempt to refresh/send config snapshot.
- If offline, mark the node as pending delivery.

#### Scenario: Offline node is marked pending
- **GIVEN** a target node is offline
- **WHEN** a WebDAV secret is distributed to it
- **THEN** the system MUST record that config delivery is pending for that node

