## ADDED Requirements

### Requirement: WebSocket Run Events Are Push-Based
The backend SHALL stream run events to connected WebSocket clients using an in-process push mechanism rather than tight database polling loops.

#### Scenario: Live events arrive without DB polling
- **WHEN** a run produces new events
- **THEN** connected WebSocket clients receive them promptly without requiring high-frequency SQLite polling per connection

### Requirement: WebSocket Run Events Are Resumable
The backend SHALL allow WebSocket clients to resume run event streaming from a known `seq` after reconnecting.

#### Scenario: Client reconnects and catches up
- **GIVEN** a client previously received events up to `seq = N`
- **WHEN** the client reconnects and requests events after `N`
- **THEN** the backend returns all events with `seq > N` in order and continues streaming new events

### Requirement: WebSocket Backpressure Triggers Resync
If a WebSocket client falls behind and misses in-process events, the backend SHALL recover by resynchronizing from SQLite without crashing the connection.

#### Scenario: Slow client triggers catch-up
- **GIVEN** a client is slow and in-process event buffers overflow
- **WHEN** the server detects lag/dropped messages
- **THEN** the server re-fetches missing events from SQLite and continues streaming from the last confirmed `seq`

### Requirement: Database Maintenance Prunes Expired Rows
The backend SHALL periodically remove rows that are expired or no longer useful, including expired sessions and enrollment tokens.

#### Scenario: Expired sessions are removed
- **WHEN** a session has `expires_at < now`
- **THEN** a periodic maintenance task deletes it from the database

### Requirement: Runs Queries Are Supported by Indexes
The backend SHALL provide appropriate database indexes for hot query paths related to run queueing, cleanup, and retention pruning.

#### Scenario: Claiming queued runs uses indexed query paths
- **WHEN** the scheduler claims the next queued run (status + time ordering)
- **THEN** the query is supported by an index that avoids scanning the full runs table

### Requirement: HTTP Layer is Modular
The backend HTTP implementation SHALL be organized into smaller modules (routes/handlers/middleware/types/errors) to keep complexity manageable.

#### Scenario: HTTP handlers live in dedicated modules
- **WHEN** adding or modifying a route
- **THEN** the handler is located in a focused module rather than a single monolithic HTTP file

### Requirement: Scheduler Avoids Idle Polling
When no work is available, the scheduler SHALL avoid tight polling loops and SHOULD prefer explicit wakeups with a safety timeout.

#### Scenario: Worker waits when idle
- **WHEN** there are no queued runs
- **THEN** the worker waits for a wakeup signal (or a long timeout) instead of polling every second

### Requirement: Graceful Shutdown Stops Background Tasks
On shutdown, the backend SHALL stop background loops (scheduler, maintenance tasks) gracefully.

#### Scenario: Shutdown cancels background loops
- **WHEN** the process receives a shutdown signal
- **THEN** background tasks exit promptly and the service shuts down cleanly

