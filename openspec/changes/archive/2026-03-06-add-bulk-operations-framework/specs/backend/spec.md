## ADDED Requirements

### Requirement: Bulk Operations Are Persisted and Processed Asynchronously
The backend SHALL provide a persistent bulk operations system that processes per-node items asynchronously.

#### Scenario: Create operation produces items
- **WHEN** a user creates a bulk operation targeting multiple nodes
- **THEN** the backend MUST persist the operation
- **AND** MUST persist one bulk item per targeted node

### Requirement: Bulk Operations Use Bounded Concurrency and Continue on Failures
The backend SHALL process bulk items with bounded concurrency and SHALL continue processing remaining items even if some items fail.

#### Scenario: Failures do not stop the bulk run
- **GIVEN** a bulk operation targets multiple nodes
- **AND** one node fails during processing
- **WHEN** the worker continues
- **THEN** other nodes MUST still be processed
- **AND** the failure MUST be recorded on the failed node item

### Requirement: Bulk Selection Supports Explicit Nodes and Label Selectors
Bulk operations SHALL support targeting nodes via:
- Explicit `node_ids[]`, or
- Label selector: `labels[]` plus `labels_mode=and|or` (default `and`)

#### Scenario: Label selector resolves nodes
- **GIVEN** multiple agents have labels
- **WHEN** a bulk operation is created using a label selector
- **THEN** the backend MUST resolve the selector to the corresponding node set

### Requirement: Bulk Operation State Is Observable via API
The backend SHALL provide authenticated APIs to list and fetch bulk operations including per-node results (status, attempts, last error, timestamps).

#### Scenario: Operator inspects results
- **WHEN** the user fetches bulk operation details
- **THEN** the response MUST include per-node statuses and error summaries

### Requirement: Bulk Operations Support Retry and Cancel
The backend SHALL support:
- Retrying failed items without re-running successful items.
- Cancelling an in-progress operation such that queued items stop being processed.

#### Scenario: Retry failed re-runs only failed items
- **GIVEN** a bulk operation has mixed success and failure items
- **WHEN** the user triggers “retry failed”
- **THEN** only failed items MUST be re-queued for processing

#### Scenario: Cancel stops queued items
- **GIVEN** a bulk operation has queued items
- **WHEN** the user cancels the operation
- **THEN** queued items MUST stop being processed

### Requirement: Authentication and CSRF Protection
Bulk operation mutation APIs (create/cancel/retry) SHALL require an authenticated session and CSRF protection.

#### Scenario: Unauthenticated user cannot create operations
- **WHEN** an unauthenticated user attempts to create a bulk operation
- **THEN** the request MUST be rejected

