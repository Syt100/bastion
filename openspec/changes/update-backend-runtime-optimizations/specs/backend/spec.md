## ADDED Requirements

### Requirement: SQLite Busy Timeout and Pool Options
The backend SHALL configure SQLite with a non-zero busy timeout and SHALL use explicit pool options to reduce transient lock failures under concurrent load.

#### Scenario: Busy timeout prevents transient lock failure
- **WHEN** concurrent operations attempt to write to SQLite
- **THEN** the backend waits up to the configured busy timeout for the lock rather than failing immediately with `database is locked`

### Requirement: Scheduler Computes Next Trigger Time
The scheduler SHALL compute the next scheduled trigger time for cron-based jobs and SHOULD avoid fixed short-interval polling when no jobs are due.

#### Scenario: Scheduler sleeps until next due time
- **GIVEN** no jobs are due for the next `T` seconds
- **WHEN** the scheduler is idle
- **THEN** it sleeps until the next due time (or until shutdown), rather than polling every few seconds

### Requirement: Notifications Loop Sleeps Until Next Due
The notifications worker SHALL sleep until the next due notification timestamp when no notifications are currently due, and SHALL wake promptly when new notifications are enqueued.

#### Scenario: Notifications worker avoids 1s polling
- **WHEN** there are no due notifications
- **THEN** the worker sleeps until the next `due_at` instead of polling every second

### Requirement: Static UI Assets Use Cache Headers and ETag
The Hub SHALL serve Web UI static assets with appropriate caching headers and SHOULD provide ETag support for conditional requests.

#### Scenario: Index is not cached
- **WHEN** the Hub serves `index.html` (including SPA fallback)
- **THEN** the response includes cache headers that prevent stale UI after upgrade

#### Scenario: Hashed assets are long-cached
- **WHEN** the Hub serves hashed build assets (e.g., under `assets/`)
- **THEN** the response includes long-lived immutable caching headers

#### Scenario: Conditional request can use ETag
- **WHEN** a client sends `If-None-Match` for an unchanged asset
- **THEN** the Hub responds with `304 Not Modified`
