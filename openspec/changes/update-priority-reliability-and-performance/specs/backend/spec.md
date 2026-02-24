## ADDED Requirements

### Requirement: Docs Filesystem Tests Are Async-Safe
The backend SHALL ensure docs filesystem-mode tests do not hold a synchronous mutex guard across asynchronous await points while preserving test isolation.

#### Scenario: Default-feature clippy checks docs tests
- **WHEN** developers run clippy with default features and warnings denied
- **THEN** docs test code passes without `await_holding_lock` violations

### Requirement: Agent WebSocket Outbox Uses Bounded Backpressure
The backend SHALL use bounded asynchronous buffering for Hub-Agent WebSocket outboxes to prevent unbounded memory growth under slow consumers.

#### Scenario: Slow peer causes outbox pressure
- **WHEN** outgoing message production exceeds peer send throughput
- **THEN** the system applies bounded backpressure/failure handling instead of unbounded queue growth

### Requirement: Agent Last-Seen Persistence Is Throttled
The backend SHALL avoid writing `agents.last_seen_at` for every incoming message from the same connection.

#### Scenario: High-frequency agent events
- **WHEN** an agent sends many events in a short interval
- **THEN** database updates for `last_seen_at` are rate-limited while connection liveness remains accurate

### Requirement: Snapshot Listing Uses Stable Keyset Pagination
The backend SHALL paginate job snapshot listings with a stable keyset cursor ordered by `(ended_at DESC, run_id DESC)`.

#### Scenario: Snapshot statuses mutate during iteration
- **WHEN** clients page through snapshots while rows change status (for example, `present` to `deleting`)
- **THEN** pagination does not skip or duplicate rows because cursor progress is keyset-based

### Requirement: Agent WS Handler Argument Fanout Is Reduced
The backend SHALL reduce high-arity argument fanout in the Agent WebSocket handling path through context grouping.

#### Scenario: Clippy lint checks handler signature
- **WHEN** clippy evaluates the optimized Agent WS handler path
- **THEN** targeted `too_many_arguments` suppression is no longer required for the refactored entrypoint
