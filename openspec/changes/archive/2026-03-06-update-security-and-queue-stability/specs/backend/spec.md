## ADDED Requirements

### Requirement: Runtime Dependency Graph Must Exclude Known Vulnerable GLib Path
The backend build configuration SHALL avoid introducing known vulnerable GLib transitive dependencies from platform-irrelevant tray implementations.

#### Scenario: Lockfile dependency graph is evaluated for Dependabot alert #7
- **WHEN** dependency metadata is generated for the repository
- **THEN** the vulnerable `glib` advisory path used by the previous tray dependency graph is no longer present

### Requirement: Offline Scheduler Queue Must Be Bounded
The agent offline scheduler SHALL use a bounded queue for pending offline run tasks.

#### Scenario: Offline task production outpaces worker consumption
- **WHEN** scheduler enqueue pressure exceeds queue capacity
- **THEN** queue memory remains bounded and enqueue failure/closure paths are handled explicitly

### Requirement: Offline Writer Command Queue Must Be Bounded
The agent offline writer SHALL use a bounded queue for event/finish commands.

#### Scenario: Offline run emits high-frequency events
- **WHEN** event writes are produced faster than persistence throughput
- **THEN** writer command buffering remains bounded with explicit full/closed handling behavior

### Requirement: Notifications Queue Listing Supports Stable Keyset Pagination
The notifications queue API SHALL support keyset pagination using `(created_at DESC, id DESC)` with an opaque cursor.

#### Scenario: Queue rows change while paging
- **WHEN** clients page through queue data during concurrent inserts/updates
- **THEN** keyset pagination avoids OFFSET-based skip/duplicate artifacts

### Requirement: Keyset List Paths Must Have Matching Composite Indexes
Storage SHALL provide composite indexes matching keyset ordering/filter predicates used by snapshot and notification listings.

#### Scenario: Large queue/snapshot datasets
- **WHEN** list APIs execute filtered keyset scans
- **THEN** query plans can use matching composite indexes for predictable latency
