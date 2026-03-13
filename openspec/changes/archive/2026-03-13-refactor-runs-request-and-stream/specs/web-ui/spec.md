## ADDED Requirements

### Requirement: Run List Dialogs SHALL Ignore Stale Responses
Run list dialogs SHALL only apply data from the latest open/load request for the current job context.

#### Scenario: User switches job quickly while run list is loading
- **GIVEN** the user opens run list for job A and immediately opens run list for job B
- **WHEN** job A's request resolves after job B
- **THEN** the dialog shows runs for job B only
- **AND** stale responses from earlier requests are ignored

### Requirement: Run Event Streams SHALL Reuse Shared Lifecycle Control
Run event consumers SHALL share the same stream lifecycle logic for connect, reconnect backoff, and sequence deduplication.

#### Scenario: WebSocket reconnect and message dedupe remains consistent
- **GIVEN** multiple run event consumers in the UI
- **WHEN** connections drop and later recover
- **THEN** both consumers use the same reconnect backoff policy
- **AND** duplicate or old event sequences are not appended
