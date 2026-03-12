# backend Specification

## Purpose
TBD - created by archiving change add-unified-cross-target-error-contract. Update Purpose after archive.
## Requirements
### Requirement: Backend SHALL emit a canonical cross-target error envelope
Backend diagnostics emitted for run failures and maintenance failures SHALL include a canonical error envelope with stable semantic fields.

#### Scenario: Target-side failure emits canonical fields
- **GIVEN** a run or maintenance step fails in a target adapter path
- **WHEN** backend appends the failure event
- **THEN** event fields SHALL include an error envelope with `schema_version`, `code`, `kind`, `retriable`, `hint`, `message`, and `transport.protocol`
- **AND** `code` and `kind` SHALL be stable, machine-readable values

### Requirement: Protocol diagnostics SHALL be transport-specific and non-HTTP-safe
Protocol-specific diagnostics SHALL be represented without forcing HTTP-only fields onto non-HTTP transports.

#### Scenario: HTTP transport failure includes HTTP status
- **GIVEN** a WebDAV or other HTTP-based target operation fails
- **WHEN** backend builds the envelope
- **THEN** `transport.protocol` SHALL be `http`
- **AND** HTTP status metadata SHALL be stored in HTTP-specific transport fields

#### Scenario: Non-HTTP transport failure excludes HTTP status
- **GIVEN** an SFTP target operation fails
- **WHEN** backend builds the envelope
- **THEN** `transport.protocol` SHALL be `sftp`
- **AND** backend SHALL NOT require an HTTP status field
- **AND** diagnostics SHALL use transport-appropriate fields (for example provider error code)

### Requirement: Retry decisions SHALL use structured retriable semantics
Retry behavior SHALL use structured envelope semantics rather than free-text message parsing as the primary source.

#### Scenario: Rate-limited failure exposes retry metadata
- **GIVEN** a target reports throttling behavior
- **WHEN** backend emits the envelope
- **THEN** `retriable.value` SHALL be `true`
- **AND** envelope SHALL include retry reason and optional retry delay metadata

### Requirement: Backend SHALL support async-operation and partial-failure diagnostics
The error envelope SHALL support asynchronous operation state and partial failure details for providers that use async APIs or batch semantics.

#### Scenario: Async provider returns accepted-then-failed operation
- **GIVEN** a cloud-drive style target reports async operation tracking
- **WHEN** an operation fails after acceptance
- **THEN** envelope context SHALL include operation metadata (for example operation id and poll hint)

#### Scenario: Batch operation partially fails
- **GIVEN** a delete or upload operation partially succeeds
- **WHEN** backend emits failure diagnostics
- **THEN** envelope context SHALL support a list of per-resource partial failures

### Requirement: Migration SHALL preserve legacy compatibility during rollout
Backend SHALL preserve existing legacy failure fields until clients are migrated.

#### Scenario: Legacy clients consume old fields during transition
- **GIVEN** a client version that still reads legacy event fields
- **WHEN** backend emits the new envelope
- **THEN** backend SHALL continue providing legacy fields during the migration window
- **AND** canonical envelope SHALL be emitted in parallel

### Requirement: WebDAV read and delete failures preserve actionable diagnostics
The backend SHALL preserve actionable transport diagnostics for WebDAV read/delete paths, including status semantics needed for classification and retry decisions.

#### Scenario: Reader receives authentication failure
- **GIVEN** WebDAV artifact read/download fails with HTTP 401 or 403
- **WHEN** the driver bridge maps the error into `DriverError`
- **THEN** the error SHALL be classified as `auth`
- **AND** the failure message SHALL include enough context to identify the HTTP status

#### Scenario: Reader receives retriable gateway failure
- **GIVEN** WebDAV read/download fails with HTTP 429 or 503
- **WHEN** retry logic evaluates the error
- **THEN** the error SHALL be treated as retriable network/upstream failure
- **AND** retry delay metadata (if present) SHALL be preserved for diagnostics

### Requirement: Driver error-kind mapping remains semantically consistent
Driver-level error mapping SHALL keep auth/config/network classes consistent with underlying WebDAV diagnostics rather than flattening all failures into `network`.

#### Scenario: Missing artifact path maps to configuration-class failure
- **GIVEN** target reader requests an invalid or missing artifact path and receives HTTP not-found semantics
- **WHEN** the bridge classifies the failure
- **THEN** it SHALL map to a non-network class (`config` or equivalent not-found configuration class)
- **AND** cleanup/run orchestration SHALL be able to block-or-handle it differently from transient network faults

### Requirement: Run failure fallback hints avoid transport-specific misdirection
When a failed run does not include explicit WebDAV PUT diagnostics, fallback hints SHALL remain actionable without assuming WebDAV transport as the root cause.

#### Scenario: Unknown non-WebDAV failure
- **GIVEN** final run failure lacks recognized transport classifiers
- **WHEN** fallback fields are generated
- **THEN** `error_kind` SHALL be `unknown`
- **AND** hint text SHALL use generic troubleshooting guidance instead of WebDAV-specific instructions

#### Scenario: Storage capacity exhaustion is detected
- **GIVEN** error-chain text indicates disk full, no space left, quota exceeded, or insufficient storage
- **WHEN** fallback fields are generated
- **THEN** `error_kind` SHALL indicate storage-capacity failure
- **AND** hint text SHALL direct operators to free capacity or adjust retention

### Requirement: Cleanup and artifact-delete events include actionable hints
Maintenance task failure events SHALL include machine-readable hints alongside error kind so operators can resolve blocked/retrying states from UI events.

#### Scenario: Cleanup task is blocked by credentials
- **GIVEN** incomplete-cleanup or artifact-delete fails with auth/config classification
- **WHEN** failure/blocked/abandoned event is appended
- **THEN** event fields SHALL include `error_kind` and `hint`
- **AND** hint SHALL describe next action (for example fixing credentials or target path configuration)

### Requirement: Rolling upload failures preserve uploader root cause
When rolling archive upload is enabled, the backend SHALL preserve uploader root-cause diagnostics when packaging observes uploader channel drop.

#### Scenario: Uploader fails before next part send
- **GIVEN** a rolling upload run and uploader task encounters a WebDAV upload error
- **WHEN** the packaging thread finalizes a subsequent part and sender `blocking_send` fails
- **THEN** the resulting run failure message includes uploader root cause details
- **AND** the message SHALL NOT degrade to only `rolling uploader dropped`

### Requirement: Packaging and uploader outcomes are reconciled deterministically
The backend SHALL always reconcile both packaging and uploader outcomes before finalizing a failed run.

#### Scenario: Packaging fails and uploader also fails
- **GIVEN** packaging path returns an error
- **AND** uploader join handle returns an error
- **WHEN** the run error is finalized
- **THEN** backend chooses a deterministic root-cause-first failure message
- **AND** preserves secondary failure details in diagnostics fields

### Requirement: Run failed events include structured diagnostics
Final run `failed` events SHALL include structured, machine-readable diagnostics fields.

#### Scenario: HTTP payload limit failure on WebDAV PUT
- **GIVEN** WebDAV PUT fails with HTTP 413 during rolling upload
- **WHEN** run terminalizes as failed
- **THEN** failed event fields include at least error code/kind, HTTP status, part metadata, and operator hint to reduce `part_size_bytes` or increase gateway limits

#### Scenario: Transport timeout or connection reset
- **GIVEN** WebDAV upload fails due to timeout/connectivity issue
- **WHEN** run terminalizes as failed
- **THEN** failed event fields include timeout/network classification and hint for timeout/retry tuning

### Requirement: Archive writer failure wrapping preserves source chain
Archive write failures SHALL preserve source error chain (instead of string-only flattening) so downstream classifiers can inspect concrete causes.

#### Scenario: Callback io::Error wraps rolling uploader diagnostic
- **GIVEN** archive write path receives callback `io::Error` containing rolling upload diagnostic
- **WHEN** archive layer maps it to `anyhow::Error`
- **THEN** classifier can still access inner cause via error chain traversal

### Requirement: WebDAV upload tuning supports timeout and retry controls
WebDAV request limits SHALL support optional timeout/retry tuning fields with validation and backward-compatible defaults.

#### Scenario: Existing job spec omits new tuning fields
- **GIVEN** a job spec without timeout/retry tuning fields
- **WHEN** job is validated and executed
- **THEN** backend uses defaults compatible with prior behavior

#### Scenario: Operator configures tighter/larger timeouts
- **GIVEN** job spec includes timeout/retry tuning values
- **WHEN** upload requests are executed
- **THEN** WebDAV client honors configured values within validated bounds

### Requirement: Runs and Operations Support Terminal Canceled Status
The backend SHALL support a terminal `canceled` lifecycle status for both runs and operations.

#### Scenario: Queued run is canceled before execution
- **GIVEN** a run is `queued`
- **WHEN** the operator requests cancellation
- **THEN** the run transitions to terminal `canceled`
- **AND** the scheduler MUST NOT start execution for that run

#### Scenario: Running operation transitions to canceled after graceful stop
- **GIVEN** an operation is `running`
- **WHEN** cancellation is requested and execution reaches a cancellation checkpoint
- **THEN** the operation performs cleanup and transitions to terminal `canceled`

### Requirement: Cancel Requests Are Persisted and Idempotent
The backend SHALL persist cancel-request metadata for runs and operations, and cancel APIs SHALL be idempotent.

#### Scenario: Repeated cancel requests return stable state
- **GIVEN** an operator has already requested cancel for a run
- **WHEN** the cancel API is called again for the same run
- **THEN** the backend returns the current status without creating conflicting transitions

### Requirement: Authenticated Cancel APIs Are Available For Runs and Operations
The backend SHALL provide authenticated mutation APIs to request cancellation for runs and operations.

#### Scenario: Operator cancels a running run
- **WHEN** an authenticated operator calls `POST /api/runs/{id}/cancel`
- **THEN** the backend records cancel intent and signals active execution to stop cooperatively

#### Scenario: Operator cancels a running restore/verify operation
- **WHEN** an authenticated operator calls `POST /api/operations/{id}/cancel`
- **THEN** the backend records cancel intent and signals active execution to stop cooperatively

### Requirement: Terminalization Is Race-Safe Against Late Results
The backend SHALL guard terminal status writes so that late success/failure results cannot overwrite `canceled`.

#### Scenario: Late success result arrives after cancellation
- **GIVEN** a run has reached terminal `canceled`
- **WHEN** a delayed success result for the same run is processed
- **THEN** the backend ignores the stale terminalization attempt
- **AND** the run remains `canceled`

### Requirement: Long-Running Work Checks Cooperative Cancellation Points
Long-running backup/restore/verify execution paths SHALL check cancellation at bounded checkpoints and exit via a cleanup-safe canceled path.

#### Scenario: Backup upload loop observes cancellation
- **GIVEN** a backup run is uploading data
- **WHEN** cancellation is requested before the next upload-part checkpoint
- **THEN** the worker exits via canceled flow after required cleanup

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

### Requirement: Validation Errors Must Expose Structured Semantics
For user-actionable validation errors, backend responses MUST keep top-level `error` and `message`, and MUST expose machine-readable semantics in `details`.

At minimum, validation responses SHALL support:
- `details.reason` for sub-cause classification under the same `error` code
- `details.field` for the primary field when a single field is involved
- `details.params` for typed parameters needed by UI localization templates

#### Scenario: Single code with multiple meanings is disambiguated by reason
- **WHEN** two validation branches share the same top-level `error` code
- **THEN** each branch includes a distinct `details.reason`
- **AND** frontend clients can distinguish branches without parsing `message`

#### Scenario: Parameterized validation includes machine-readable params
- **WHEN** a validation rule uses thresholds (such as min/max length)
- **THEN** the response includes threshold values in `details.params`
- **AND** the values can be used for localized UI templates

### Requirement: Backend Supports Multi-Field Validation Violations
When one request has multiple field validation failures, backend responses MUST support a structured violations list.

The violations list SHALL support per-item:
- `field`
- `reason`
- optional `params`
- optional human-readable `message`

#### Scenario: Multiple field failures are returned without lossy flattening
- **WHEN** request validation detects more than one field failure
- **THEN** the response includes `details.violations[]`
- **AND** each violation includes at least `field` and `reason`

### Requirement: Agent List Error Transport Must Be Structured
Filesystem/WebDAV browse errors relayed from agents to hub MUST include machine-readable error codes.

For agent-originated list errors:
- the transport SHALL include `error_code`
- the hub SHALL map known agent codes to stable API error codes without relying on message substrings
- message text remains a fallback for unknown legacy cases only

#### Scenario: Filesystem list not-directory maps from structured code
- **WHEN** agent returns a filesystem list failure with a machine-readable not-directory code
- **THEN** hub API responds with stable `error = "not_directory"`
- **AND** this mapping does not depend on `message` text

#### Scenario: Legacy agent message still falls back safely
- **WHEN** an older agent does not provide structured error code
- **THEN** hub still returns a meaningful API error using compatibility fallback
- **AND** newer structured mapping takes precedence when available

### Requirement: Structured Contract Remains Backward Compatible
The backend SHALL keep the existing top-level API error envelope (`error`, `message`, `details`) while introducing structured detail fields.

#### Scenario: Existing clients that only read error/message continue to work
- **WHEN** a client ignores `details.reason` and `details.params`
- **THEN** the client still receives valid `error` and `message`
- **AND** behavior remains backward compatible

### Requirement: Open Dependabot Alerts MUST Be Remediated with Verified Dependency Constraints
Repository dependency updates MUST remediate active Dependabot alerts using manifest/lockfile changes that are validated in CI.

#### Scenario: Rust lockfile contains vulnerable crate patch level
- **WHEN** Rust dependency advisories affect versions in `Cargo.lock`
- **THEN** workspace manifests and lockfiles are updated to patched versions or constrained to avoid vulnerable transitive paths
- **AND** Rust build/test checks continue to pass

#### Scenario: npm transitive vulnerability is reported
- **WHEN** npm alerts are raised for transitive packages in UI/docs lockfiles
- **THEN** manifests apply explicit override constraints to patched versions when needed
- **AND** lockfiles are regenerated and validated with existing build/test quality gates

### Requirement: Dependency Surface MUST Exclude Unused High-Risk Feature Paths
Workspace dependency features MUST disable unused database/runtime stacks that increase vulnerability exposure.

#### Scenario: SQLx default feature set includes unused drivers
- **WHEN** runtime only requires SQLite paths
- **THEN** workspace dependency configuration disables SQLx default features and enables only required features in member crates
- **AND** lockfile no longer carries unused vulnerable crypto/database subgraphs introduced solely by defaults

### Requirement: Test Credentials MUST Avoid Hard-Coded Secret Literals
Backend test code MUST avoid static credential/password literals that are interpreted as hard-coded cryptographic values by security scanners.

#### Scenario: Auth test setup needs a password
- **WHEN** a test creates a user/session requiring a password
- **THEN** it uses a runtime-generated passphrase value instead of a hard-coded literal

#### Scenario: Keypack tests need import/export password
- **WHEN** tests call keypack export/import helpers with a password
- **THEN** they use generated passphrase values, including wrong-password branches

### Requirement: Secret-Bearing Tests MUST Avoid Cleartext Value Emission
Backend tests that handle decrypted or secret-bearing values MUST not format those values into logs/panic/debug output.

#### Scenario: Secret equality assertion fails
- **WHEN** a secret-bearing assertion fails
- **THEN** the failure message does not include raw secret bytes/string values

#### Scenario: Unexpected enum variant in secret flow
- **WHEN** tests guard secret-bearing enum branches
- **THEN** panic/error messages avoid `{:?}` dumps that could include sensitive fields

### Requirement: Effective Client IP Uses Trusted-Hop Forwarded Chain Parsing
The backend SHALL derive effective client IP behind trusted proxies by walking forwarded hops from right to left, removing trusted proxy hops, and selecting the first untrusted hop.

#### Scenario: Spoofed left-most forwarded IP does not bypass throttling key
- **GIVEN** a trusted reverse proxy forwards a request with attacker-controlled left-most `X-Forwarded-For`
- **WHEN** the backend derives effective client IP
- **THEN** it does not blindly trust the left-most entry
- **AND** login throttling keys remain bound to the effective untrusted client hop

#### Scenario: Invalid forwarded chain falls back safely
- **WHEN** forwarded headers are malformed or cannot be parsed
- **THEN** the backend falls back to peer IP

### Requirement: Run Events WebSocket Origin Validation Compares Full Origin Tuple
The backend SHALL validate run-events WebSocket origin using effective scheme, host, and port.

#### Scenario: Host matches but port differs
- **WHEN** request host matches but origin port differs from effective request port
- **THEN** the backend rejects the connection as invalid origin

#### Scenario: Host matches but scheme differs
- **WHEN** request host matches but origin scheme differs from effective request scheme
- **THEN** the backend rejects the connection as invalid origin

### Requirement: Setup Initialization Is Atomic
The backend SHALL enforce atomic first-user initialization so concurrent setup requests cannot initialize more than once.

#### Scenario: Concurrent setup requests race
- **WHEN** multiple setup requests are processed concurrently
- **THEN** at most one request succeeds
- **AND** all others receive a conflict-style error

### Requirement: Backend Enforces Authentication Input Policy
The backend SHALL enforce input policy for auth/setup credentials regardless of client behavior.

#### Scenario: Blank username is rejected
- **WHEN** setup or auth receives an empty or whitespace-only username
- **THEN** the backend returns a validation error

#### Scenario: Too-short password is rejected
- **WHEN** setup receives a password below minimum policy length
- **THEN** the backend returns a validation error

### Requirement: Agents List API Supports Server-Side Pagination and Search/Status Filtering
The agents list API MUST support server-side pagination and filtering for status/search while preserving existing label filtering semantics.

#### Scenario: Agents list returns paged payload with metadata
- **GIVEN** an authenticated request to `/api/agents`
- **WHEN** the caller provides `page` and `page_size`
- **THEN** the response includes `items`, `page`, `page_size`, and `total`
- **AND** `items` contains only rows for the requested page

#### Scenario: Agents list filters by status and search query
- **GIVEN** agents with mixed online/offline/revoked states and distinct names/ids
- **WHEN** the caller provides `status` and `q`
- **THEN** only matching agents are returned
- **AND** `total` reflects the filtered count before pagination

### Requirement: Jobs List API Supports Server-Side Node-Scoped Filtering, Sorting, and Pagination
The jobs list API MUST support server-side filters for node scope, include-archived mode, latest-run status, schedule mode, and free-text search, plus deterministic sorting and page/page_size pagination.

#### Scenario: Jobs list returns one page with applied filters
- **GIVEN** an authenticated request to `/api/jobs`
- **WHEN** the caller specifies `node_id`, `include_archived`, `latest_status`, `schedule_mode`, `q`, `page`, and `page_size`
- **THEN** only matching jobs are returned in `items`
- **AND** `total` reports filtered result size before pagination

#### Scenario: Jobs list honors remote sort key
- **GIVEN** jobs with different names and update timestamps
- **WHEN** the caller provides a supported `sort` value
- **THEN** rows are ordered according to that sort mode
- **AND** ordering is stable for equal sort keys

### Requirement: Critical Background Tasks Are Supervised
The backend SHALL supervise critical long-running background tasks so that unexpected task panics are detected rather than failing silently.

#### Scenario: Panic in a critical task is detected
- **GIVEN** a critical background task panics unexpectedly
- **WHEN** the panic occurs
- **THEN** the backend emits an error log identifying the task

### Requirement: Panic In A Critical Task Triggers Graceful Shutdown
When a supervised critical background task panics unexpectedly, the backend MUST trigger graceful shutdown via the shared cancellation token.

#### Scenario: Panic cancels shutdown token
- **GIVEN** the Hub is running normally
- **WHEN** a supervised critical background task panics unexpectedly
- **THEN** the shared shutdown token is cancelled so the server shuts down gracefully

### Requirement: RunEventsBus Does Not Panic On Poisoned Mutex
The backend MUST tolerate `RunEventsBus` mutex poisoning and MUST continue operating without panicking when the bus lock is poisoned.

#### Scenario: Publish continues after a poisoned lock
- **GIVEN** a prior panic poisoned the RunEventsBus mutex
- **WHEN** the backend publishes a run event
- **THEN** publishing does not panic and the bus remains usable

#### Scenario: Subscribe continues after a poisoned lock
- **GIVEN** a prior panic poisoned the RunEventsBus mutex
- **WHEN** the backend subscribes to a run event stream
- **THEN** subscribing does not panic and a receiver is returned

### Requirement: Dashboard Overview API
The backend SHALL provide an authenticated API endpoint to return a dashboard overview payload for the Web UI.

#### Scenario: Auth is required
- **WHEN** a client requests `GET /api/dashboard/overview` without a valid session
- **THEN** the server responds with `401 Unauthorized`

#### Scenario: The response includes a 7-day trend series
- **WHEN** a client requests `GET /api/dashboard/overview` with a valid session
- **THEN** the response includes a 7-day trend series for success/failed runs

#### Scenario: The endpoint is safe
- **WHEN** the endpoint returns recent runs and summary stats
- **THEN** it MUST NOT include secret values (credentials, tokens, encryption keys)

### Requirement: Persist Run Stage Boundaries From Progress Snapshots
The backend SHALL persist stage boundaries for progress snapshots as run events.

#### Scenario: Progress snapshot stage transition emits a run event
- **GIVEN** a run receives progress snapshots with stage values
- **WHEN** the stage value changes (e.g. `scan` → `packaging` → `upload`)
- **THEN** the backend records a run event for the new stage

#### Scenario: Identical stage snapshots do not emit duplicate run events
- **GIVEN** a run receives progress snapshots with a stage value
- **WHEN** multiple snapshots arrive with the same stage value
- **THEN** the backend does not record duplicate stage events for that stage

### Requirement: Backup Progress Snapshots Include Stable Source Totals
The backend SHALL include stable SOURCE totals (files, dirs, bytes) for filesystem backup runs in the run progress snapshot detail when the totals are known, and SHALL keep these totals visible across stage transitions.

#### Scenario: Source totals remain visible after entering upload
- **GIVEN** a filesystem backup run computes SOURCE totals during scan or packaging
- **WHEN** the run transitions into the upload stage
- **THEN** subsequent run progress snapshots include the previously computed SOURCE totals

### Requirement: raw_tree_v1 Upload Reports A Stable Transfer Total
For filesystem backups using raw_tree_v1, the backend SHALL expose a stable TRANSFER total bytes during upload so the UI can compute a meaningful percentage.

#### Scenario: Upload total bytes is stable for raw_tree_v1
- **GIVEN** a filesystem backup run uses raw_tree_v1
- **WHEN** the run is uploading artifacts to the target
- **THEN** the run progress snapshot includes TRANSFER total bytes that do not grow during the upload

### Requirement: Backup Upload Snapshots Include Transfer Metrics
During upload, the backend SHALL include TRANSFER done bytes and TRANSFER total bytes in the run progress snapshot detail for filesystem backup runs.

#### Scenario: Transfer done bytes increases during upload
- **GIVEN** a filesystem backup run is in the upload stage
- **WHEN** the system uploads additional data to the target
- **THEN** TRANSFER done bytes in the run progress snapshot increases until it reaches TRANSFER total bytes

### Requirement: Operations May Reference a Subject Entity
The backend SHALL support linking an operation to a domain entity via a subject reference (`subject_kind`, `subject_id`).

#### Scenario: Operation has a run subject
- **WHEN** an operation is created for a run-scoped action
- **THEN** the operation stores `subject_kind = "run"` and `subject_id = <run_id>`

### Requirement: Restore and Verify Operations Link Back to the Run
When a restore or verify operation is started from a run, the backend SHALL create the operation linked to the run.

#### Scenario: Restore started from a successful run
- **GIVEN** a run with `status = success`
- **WHEN** the user starts a restore operation for the run
- **THEN** the created operation is linked to the run subject

#### Scenario: Verify started from a successful run
- **GIVEN** a run with `status = success`
- **WHEN** the user starts a verify operation for the run
- **THEN** the created operation is linked to the run subject

### Requirement: Run-Scoped Operations Listing API
The backend SHALL provide an API to list operations linked to a run.

#### Scenario: List operations for a run
- **GIVEN** a run with linked operations
- **WHEN** the user requests `GET /api/runs/{run_id}/operations`
- **THEN** the backend returns operations linked to the run ordered by `started_at` descending

### Requirement: Run Read API
The backend SHALL provide an authenticated API to fetch run details by id.

#### Scenario: Fetch an existing run
- **WHEN** the user requests `GET /api/runs/{run_id}`
- **THEN** the response includes run status, timestamps, job id, and any available summary/error fields

#### Scenario: Run not found
- **WHEN** the user requests `GET /api/runs/{run_id}` for an unknown id
- **THEN** the backend returns `404 run_not_found`

### Requirement: Persist Latest Progress Snapshot For Runs
The backend SHALL persist the latest progress snapshot for a running backup run and expose it via authenticated run APIs.

#### Scenario: Run progress is readable while running
- **GIVEN** a run is `running`
- **WHEN** the system updates its progress snapshot
- **THEN** subsequent reads of the run include the latest `progress` snapshot

### Requirement: Persist Latest Progress Snapshot For Operations
The backend SHALL persist the latest progress snapshot for a running operation (restore/verify) and expose it via authenticated operation APIs.

#### Scenario: Operation progress is readable while running
- **GIVEN** an operation is `running`
- **WHEN** the system updates its progress snapshot
- **THEN** subsequent reads of the operation include the latest `progress` snapshot

### Requirement: Filesystem Backup Supports Optional Pre-Scan
Filesystem backup jobs SHALL support an optional pre-scan stage (`filesystem.source.pre_scan`) to compute totals for progress and ETA. The default value SHALL be `true`.

#### Scenario: Pre-scan computes totals
- **GIVEN** a filesystem job has `source.pre_scan = true`
- **WHEN** a run starts
- **THEN** the run enters a scan stage and computes totals before packaging/upload

#### Scenario: Pre-scan can be disabled
- **GIVEN** a filesystem job has `source.pre_scan = false`
- **WHEN** a run starts
- **THEN** the backend proceeds without requiring totals to be present in the progress snapshot

### Requirement: Progress Updates Are Throttled
The backend SHALL throttle progress snapshot writes and broadcasts to avoid excessive database writes and event volume while still remaining responsive.

#### Scenario: Throttling limits update frequency
- **WHEN** a long-running backup or restore is executing
- **THEN** progress updates are emitted no more than once per second per run/operation (except for stage changes)

### Requirement: WebDAV Browsing Uses Stored Credentials and Does Not Leak Secrets
When listing WebDAV directories for browsing, the backend SHALL use stored WebDAV credentials and SHALL NOT log secret payload values.

#### Scenario: Logs redact credentials
- **WHEN** the system performs a WebDAV list operation
- **THEN** logs may include a redacted URL but MUST NOT include the WebDAV password

### Requirement: Age Identity Distribution Is Auditable and Does Not Leak Secrets
When distributing backup age identities to Agents, the backend SHALL record audit-friendly events and SHALL NOT log secret payload values.

#### Scenario: Logs contain key names but not key values
- **WHEN** an age identity secret `backup_age_identity/K` is distributed
- **THEN** logs/events may reference `K` and the target Agent
- **AND** logs/events MUST NOT include the identity private key contents

### Requirement: Restore Uses a Streaming Engine with Pluggable Sources and Sinks
The backend SHALL implement restore via a streaming restore engine that consumes a pluggable artifact source and writes to a pluggable restore sink, to enable future restore destinations while preserving current restore semantics.

#### Scenario: Restore logic is decoupled from storage backends
- **WHEN** a developer adds a new restore destination backend
- **THEN** they implement a new restore sink without modifying archive parsing logic

### Requirement: Restore Refactor Preserves Existing Behavior
The backend SHALL preserve current restore behavior (selection filtering, conflict policy semantics, and operation events) while refactoring restore internals.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Bulk Job Deploy Clones a Source Job to Target Nodes
The backend SHALL support a bulk job deploy operation that clones an existing source job to multiple target nodes selected by explicit ids or label selectors.

#### Scenario: Jobs are created for each target node
- **GIVEN** a source job exists
- **WHEN** the operator deploys the job to multiple target nodes
- **THEN** the backend MUST create a corresponding job for each targeted node

### Requirement: Name Template Default and Collision Handling
Bulk job deploy SHALL use a name template with a default that includes the node id (e.g., `"{name} ({node})"`). If a generated name still collides, the backend SHALL automatically disambiguate with a suffix.

#### Scenario: Default name includes node id
- **WHEN** the operator does not override the name template
- **THEN** each deployed job name MUST include the target node id

#### Scenario: Collision is auto-suffixed
- **GIVEN** a generated job name already exists on a node
- **WHEN** the deploy operation attempts to create the job
- **THEN** the backend MUST adjust the name to avoid ambiguity (e.g., add `#2`)

### Requirement: Per-node Preflight Validation and Clear Errors
The backend SHALL validate prerequisites per node (e.g., node-scoped secrets referenced by the job spec) and SHALL record failures with clear, actionable error summaries.

#### Scenario: Missing credential fails only that node
- **GIVEN** the source job references a WebDAV credential name
- **AND** a target node does not have that credential
- **WHEN** the deploy operation runs
- **THEN** that node MUST fail with a clear error message
- **AND** other nodes MUST continue processing

### Requirement: Preview Before Execution
The backend SHALL support a preview capability that returns, per node, the planned job name and validation outcome prior to execution.

#### Scenario: Preview returns planned names and validation
- **WHEN** the operator requests a deploy preview
- **THEN** the preview MUST include planned names and per-node validation results

### Requirement: Hub Persists Desired and Applied Config Snapshot IDs Per Agent
The backend SHALL persist per-agent config synchronization state including the desired snapshot id and the last applied (acknowledged) snapshot id.

#### Scenario: Desired vs applied is recorded
- **GIVEN** an agent exists
- **WHEN** the Hub sends a config snapshot for that agent
- **THEN** the agent record MUST reflect the desired snapshot id
- **AND** the applied snapshot id MUST be updated only after a matching `ConfigAck`

### Requirement: ConfigAck Updates Applied Snapshot State
When the Hub receives an agent `ConfigAck`, the backend SHALL persist the acked snapshot id as the agent’s applied snapshot id along with a timestamp.

#### Scenario: Ack makes an agent “synced”
- **GIVEN** an agent has a desired snapshot id `S`
- **WHEN** the agent sends `ConfigAck` for snapshot `S`
- **THEN** the agent MUST be considered “synced” by the backend

### Requirement: Offline Nodes Are Supported Without Losing Desired State
If a node is offline when configuration changes, the backend SHALL still update the desired snapshot id and SHALL deliver the snapshot on reconnect.

#### Scenario: Offline node becomes pending then syncs on reconnect
- **GIVEN** an agent is offline
- **WHEN** a configuration change occurs that affects the agent
- **THEN** the backend MUST update the desired snapshot id and record that the node is pending delivery
- **AND** **WHEN** the agent reconnects
- **THEN** the Hub MUST send the desired snapshot

### Requirement: API Surfaces Config Sync Status
The backend SHALL expose sync state in authenticated agent list/detail APIs, including:
- desired snapshot id
- applied snapshot id
- sync status (synced/pending/error/offline)
- last sync attempt time and error summary (if any)

#### Scenario: Operator can view sync status via API
- **WHEN** the user lists agents
- **THEN** the response MUST include per-agent sync status fields

### Requirement: Operator Can Trigger “Sync Config Now” (Single and Bulk)
The backend SHALL provide an operator action to trigger sending the latest desired config snapshot to:
- a single agent, and
- multiple agents via bulk operations

#### Scenario: Sync now sends snapshot or records offline
- **WHEN** the operator triggers “sync now” for an online agent
- **THEN** the Hub MUST attempt to send the latest config snapshot
- **AND** **WHEN** the operator triggers “sync now” for an offline agent
- **THEN** the system MUST record the node as pending delivery without failing the entire operation

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

### Requirement: System Status Includes Build Metadata
The backend SHALL expose build metadata via `/api/system` so the Web UI can display Hub version and build time.

#### Scenario: System status includes build time
- **WHEN** a client requests `/api/system`
- **THEN** the response includes `version` and `build_time_unix`

#### Scenario: Source build without git still works
- **GIVEN** the Hub is built from source without a `.git` directory
- **WHEN** a client requests `/api/system`
- **THEN** the endpoint still returns successfully
- **AND** build metadata may fall back to `unknown`

### Requirement: Jobs Persist Schedule Timezone
The backend SHALL persist an IANA `schedule_timezone` per job, and SHALL expose it via authenticated job APIs.

#### Scenario: Create a job with explicit timezone
- **WHEN** a job is created with `schedule_timezone=Asia/Shanghai`
- **THEN** subsequent reads of the job return the same `schedule_timezone`

### Requirement: Hub Exposes Default Timezone
The backend SHALL expose the Hub timezone via `/api/system` so the UI can default new job schedules.

#### Scenario: System status includes hub timezone
- **WHEN** a user requests `/api/system`
- **THEN** the response includes `hub_timezone`

### Requirement: Timezone-Aware Scheduling with DST Semantics
The scheduler SHALL interpret cron schedules in the job’s configured timezone, using wall-clock semantics.

#### Scenario: DST gap is skipped
- **GIVEN** a timezone with a DST spring-forward gap
- **WHEN** the missing local time would be due by cron
- **THEN** no run is enqueued for the nonexistent local time

#### Scenario: DST fold runs once
- **GIVEN** a timezone with a DST fall-back fold
- **WHEN** a cron is due at a repeated local time
- **THEN** exactly one run is enqueued for that local wall time

### Requirement: Cron Validation Avoids “Never Triggers”
The backend SHALL validate cron expressions and reject schedules that could never trigger under the minute-based scheduler loop.

#### Scenario: 6-field cron with seconds != 0 is rejected
- **WHEN** a user sets a 6-field cron where seconds is not `0`
- **THEN** the API returns `invalid_schedule`

### Requirement: Agent Offline Scheduling Uses Same Timezone Semantics
Agent config snapshots SHALL include `schedule_timezone` and offline scheduling SHALL use the same evaluation logic as the Hub scheduler.

#### Scenario: Snapshot includes schedule timezone
- **WHEN** the Hub sends a config snapshot for an agent job
- **THEN** the job config includes `schedule_timezone`

### Requirement: Incomplete Cleanup Uses a Persistent Queue
The backend SHALL track stale incomplete run cleanup via a persistent task queue with retry scheduling to avoid tight loops and log spam.

#### Scenario: Unreachable targets do not cause tight loops
- **GIVEN** a run with an unreachable WebDAV target
- **WHEN** the cleanup worker processes due tasks
- **THEN** the task is retried with backoff and does not repeatedly emit warnings in a tight loop

### Requirement: Cleanup Tasks Are Observable and Operable
The backend SHALL expose authenticated APIs to list cleanup tasks, view attempt history, and perform operator actions (retry/ignore/unignore).

#### Scenario: Operator can retry now
- **WHEN** a user triggers “retry now” for a cleanup task
- **THEN** the task becomes due immediately and the cleanup worker attempts it again

### Requirement: Runs Store Target Snapshots
The backend SHALL persist a per-run target snapshot for maintenance workflows.

#### Scenario: Cleanup uses the run snapshot
- **GIVEN** a job target configuration changes after a run starts
- **WHEN** maintenance performs cleanup for that run
- **THEN** maintenance uses the stored run target snapshot rather than the job’s current target

### Requirement: Jobs Support Archive (Soft Delete)
The backend SHALL allow jobs to be archived (soft-deleted) so history remains, while permanent deletion remains available and cascades by default.

#### Scenario: Archive keeps history
- **WHEN** a user archives a job
- **THEN** the job is hidden from default listings and no new runs are scheduled, but past runs remain queryable

### Requirement: Scheduler Worker Loop Is Split Into Focused Submodules
The backend SHALL organize scheduler worker loop phases into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Phase logic is easier to modify safely
- **WHEN** a developer adjusts agent dispatch/polling behavior or local execution completion logic
- **THEN** changes are localized to the corresponding submodule rather than requiring edits across one large loop function

### Requirement: Scheduler Worker Loop Refactor Preserves Behavior
The backend SHALL preserve existing scheduling/dispatch/notification behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: WebDAV Secret Handlers Use Shared Helpers
The backend SHALL reduce duplication in WebDAV secret HTTP handlers by extracting shared validation and persistence helpers, without changing behavior.

#### Scenario: Changes are localized
- **WHEN** a developer adjusts WebDAV secret validation or persistence behavior
- **THEN** changes primarily occur in shared helper functions rather than being duplicated across hub-level and node-level handlers

### Requirement: WebDAV Secret Helper Refactor Preserves Behavior
The backend SHALL preserve existing WebDAV secret CRUD behavior while refactoring internal helper structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Job Spec Code Is Split Into Focused Submodules
The backend SHALL organize job spec parsing and type definitions into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Validation changes are localized
- **WHEN** a developer needs to adjust job spec validation rules
- **THEN** changes are localized to the validation submodule and do not require edits to type definitions

### Requirement: Job Spec Refactor Preserves Behavior
The backend SHALL preserve existing job spec parsing/validation behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: WebDAV Client Uses Shared Request Helpers
The backend SHALL reduce duplication in the WebDAV client by extracting shared request helper logic, without changing behavior.

#### Scenario: Auth wiring changes are localized
- **WHEN** a developer needs to adjust how WebDAV requests apply authentication
- **THEN** the change is made in a shared helper rather than duplicated across multiple request methods

### Requirement: WebDAV Client Helper Refactor Preserves Behavior
The backend SHALL preserve existing WebDAV client behavior while refactoring internal helper structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Scheduler Worker Execute Logic Is Split By Job Type
The backend SHALL organize scheduler worker execute logic into focused Rust modules per job type to improve maintainability, without changing behavior.

#### Scenario: Job-type execution changes are localized
- **WHEN** a developer needs to adjust how a specific job type is packaged or uploaded
- **THEN** changes are primarily localized to the corresponding job-type execute module

### Requirement: Scheduler Worker Execute Refactor Preserves Behavior
The backend SHALL preserve existing scheduler worker execution behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Restore Operations Code Is Split Into Focused Submodules
The backend SHALL organize restore operations code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Restore and verify logic are separated
- **WHEN** a developer needs to adjust restore or verify behavior
- **THEN** changes are localized to the corresponding restore/verify submodule and do not require edits to the other flow

### Requirement: Restore Operations Refactor Preserves Behavior
The backend SHALL preserve existing restore operations behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Jobs CRUD Handlers Use Shared Validation Helpers
The backend SHALL reduce duplication in jobs CRUD HTTP handlers by extracting shared validation/normalization helpers, without changing behavior.

#### Scenario: Validation changes are localized
- **WHEN** a developer needs to adjust jobs CRUD validation behavior
- **THEN** changes primarily occur in shared helper functions rather than being duplicated across multiple handlers

### Requirement: Jobs CRUD Helper Refactor Preserves Behavior
The backend SHALL preserve existing jobs CRUD behavior while refactoring internal helper structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Restore Entries Index Code Is Split Into Focused Submodules
The backend SHALL organize restore entries index code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Fetching and listing logic are separated
- **WHEN** a developer needs to adjust entries index caching or download behavior
- **THEN** changes are localized to the fetch/cache submodule and do not require edits to listing/filtering code

### Requirement: Restore Entries Index Refactor Preserves Behavior
The backend SHALL preserve existing restore entries index behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: WebDAV Helpers Are Centralized
The backend SHALL avoid duplicating WebDAV helper logic by centralizing shared helpers, without changing behavior.

#### Scenario: URL redaction logic is consistent
- **WHEN** WebDAV code logs a URL for diagnostics
- **THEN** the same redaction logic is applied across both WebDAV storage and WebDAV client code paths

### Requirement: WebDAV Helper Refactor Preserves Behavior
The backend SHALL preserve existing WebDAV behavior while refactoring internal helper structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Agent Offline Entrypoint Uses Directory Module Layout
The backend SHALL organize agent offline code so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Offline navigation is consistent
- **WHEN** a developer needs to inspect offline scheduling or sync behavior
- **THEN** the entrypoint and its offline submodules are found under the same `offline/` directory

### Requirement: Agent Offline Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing agent offline behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: HTTP Submodule Entrypoints Use Directory Module Layout
The backend SHALL organize HTTP submodules so that each entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: HTTP handler navigation is consistent
- **WHEN** a developer needs to inspect HTTP request handling for agents/jobs/notifications/secrets
- **THEN** the entrypoint and related submodules are found under the same subdirectory

### Requirement: HTTP Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing HTTP behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Notifications Entrypoint Uses Directory Module Layout
The backend SHALL organize the notifications module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Notifications navigation is consistent
- **WHEN** a developer needs to inspect notification logic
- **THEN** the entrypoint and its notifications submodules are found under the same `notifications/` directory

### Requirement: Notifications Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing notifications behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Scheduler Entrypoint Uses Directory Module Layout
The backend SHALL organize the scheduler module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Scheduler navigation is consistent
- **WHEN** a developer needs to inspect scheduling logic
- **THEN** the scheduler entrypoint and its submodules are found under the same `scheduler/` directory

### Requirement: Scheduler Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing scheduler behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Filesystem Tar Entrypoint Uses Directory Module Layout
The backend SHALL organize the filesystem tar module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Tar code navigation is consistent
- **WHEN** a developer needs to inspect tar packaging logic
- **THEN** the entrypoint and its tar-related submodules are found under the same `tar/` directory

### Requirement: Filesystem Tar Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing filesystem tar behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Backup Filesystem Entrypoint Uses Directory Module Layout
The backend SHALL organize the backup filesystem module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Filesystem backup navigation is consistent
- **WHEN** a developer needs to inspect filesystem backup building logic
- **THEN** the entrypoint and its related modules are found under the same `filesystem/` directory

### Requirement: Backup Filesystem Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing filesystem backup behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Scheduler Worker Entrypoint Uses Directory Module Layout
The backend SHALL organize the scheduler worker module so that the entrypoint lives alongside its submodules in a directory module layout, without changing behavior.

#### Scenario: Worker navigation is consistent
- **WHEN** a developer needs to inspect how runs are processed
- **THEN** the worker entrypoint and its submodules are found under the same `worker/` directory

### Requirement: Scheduler Worker Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing scheduler worker behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Jobs Repo Code Is Split Into Focused Submodules
The backend SHALL organize jobs repository code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Overlap policy changes are isolated from DB queries
- **WHEN** a developer needs to update `OverlapPolicy` parsing/serialization
- **THEN** the change primarily occurs in the jobs types submodule and DB query logic remains localized in the repo submodule

### Requirement: Jobs Repo Refactor Preserves Behavior
The backend SHALL preserve existing jobs repository behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Operations Repo Code Is Split Into Focused Submodules
The backend SHALL organize operations repository code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Type changes are isolated from DB queries
- **WHEN** a developer needs to add a new `OperationKind`
- **THEN** the primary change occurs in the operations types submodule and DB query logic remains localized in the repo submodule

### Requirement: Operations Repo Refactor Preserves Behavior
The backend SHALL preserve existing operations repository behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Storage Auth Code Is Split Into Focused Submodules
The backend SHALL organize storage auth code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Login throttling changes are localized
- **WHEN** a developer needs to adjust login throttling behavior
- **THEN** the change primarily occurs in the throttle submodule and does not require edits to password hashing or session code

### Requirement: Storage Auth Refactor Preserves Behavior
The backend SHALL preserve existing auth behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Offline Sync Code Is Split Into Focused Submodules
The backend SHALL organize agent offline sync code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Event loading changes are localized
- **WHEN** a developer needs to adjust how offline run events are parsed from `events.jsonl`
- **THEN** the change primarily occurs in the events-loader submodule and does not require edits to HTTP ingest logic

### Requirement: Offline Sync Refactor Preserves Behavior
The backend SHALL preserve existing offline sync behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Offline Storage Code Is Split Into Focused Submodules
The backend SHALL organize agent offline storage code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Writer changes are localized
- **WHEN** a developer needs to adjust how offline run events are appended
- **THEN** the change primarily occurs in the writer/IO submodules and does not require edits to on-disk type definitions

### Requirement: Offline Storage Refactor Preserves Behavior
The backend SHALL preserve existing offline storage behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Logging Code Is Split Into Focused Submodules
The backend SHALL organize logging code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Pruning logic changes are localized
- **WHEN** a developer needs to adjust rotated-log pruning behavior
- **THEN** the change primarily occurs in the pruning submodule and does not require edits to initialization/filtering code

### Requirement: Logging Refactor Preserves Behavior
The backend SHALL preserve existing logging behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Agent Tasks Code Is Split Into Focused Submodules
The backend SHALL organize agent task handling code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Per-spec backup logic changes are localized
- **WHEN** a developer needs to adjust filesystem backup behavior
- **THEN** the change primarily occurs in the filesystem task submodule and does not require edits to sqlite or vaultwarden handlers

### Requirement: Agent Tasks Refactor Preserves Behavior
The backend SHALL preserve existing agent task handling behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Agent Connect Code Is Split Into Focused Submodules
The backend SHALL organize agent websocket connect code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Message handling changes are localized
- **WHEN** a developer needs to adjust how a specific hub message is processed
- **THEN** the change primarily occurs in the message-handlers submodule and does not require edits to handshake or heartbeat logic

### Requirement: Agent Connect Refactor Preserves Behavior
The backend SHALL preserve existing agent websocket connect behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Agent Managed State Code Is Split Into Focused Submodules
The backend SHALL organize agent managed state code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Task result persistence changes are localized
- **WHEN** a developer needs to adjust how task results are cached on disk
- **THEN** the change primarily occurs in the task-results submodule and does not require edits to managed snapshot encryption logic

### Requirement: Agent Managed State Refactor Preserves Behavior
The backend SHALL preserve existing agent managed behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Backup Restore Entrypoint Is Kept Focused
The backend SHALL keep the restore module entrypoint focused by moving tests and implementation details into dedicated modules, without changing behavior.

#### Scenario: Restore types remain accessible
- **WHEN** a caller uses `bastion_backup::restore::ConflictPolicy` or `bastion_backup::restore::RestoreSelection`
- **THEN** the types remain available with the same paths and semantics after refactoring

### Requirement: Restore Entrypoint Refactor Preserves Behavior
The backend SHALL preserve existing restore behavior while refactoring restore module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Agent Offline Scheduler Code Is Split Into Focused Submodules
The backend SHALL organize agent offline scheduler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Cron logic changes are localized
- **WHEN** a developer needs to change offline cron scheduling behavior
- **THEN** the change primarily occurs in the cron-loop submodule and does not require edits to worker execution or sink parsing logic

### Requirement: Agent Offline Scheduler Refactor Preserves Behavior
The backend SHALL preserve existing offline scheduler behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: HTTP UI Fallback Code Is Isolated From Router Setup
The backend SHALL isolate UI fallback asset-serving logic into a focused module to improve maintainability, without changing behavior.

#### Scenario: UI cache header changes are localized
- **WHEN** a developer needs to adjust cache-control or ETag behavior for UI assets
- **THEN** the change primarily occurs in the UI fallback module and does not require edits to API routing setup

### Requirement: HTTP UI Fallback Refactor Preserves Behavior
The backend SHALL preserve UI fallback behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Vaultwarden Backup Code Is Split Into Focused Submodules
The backend SHALL organize vaultwarden backup code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Hashing logic is localized
- **WHEN** a developer needs to adjust hashing behavior or file IO helpers
- **THEN** the change primarily occurs in the hashing/IO submodule and does not require edits to tar walking logic

### Requirement: Vaultwarden Backup Refactor Preserves Behavior
The backend SHALL preserve existing vaultwarden backup behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Filesystem Tar Walking Code Is Split Into Focused Submodules
The backend SHALL organize filesystem tar walking code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Legacy root logic is localized
- **WHEN** a developer needs to adjust legacy `filesystem.source.root` handling
- **THEN** the change primarily occurs in the legacy-root submodule and does not require edits to selected-path walking logic

### Requirement: Filesystem Tar Walk Refactor Preserves Behavior
The backend SHALL preserve existing filesystem tar walking behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Storage Secrets Code Is Split Into Focused Submodules
The backend SHALL organize storage secrets (keyring/crypto/keypack) code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Keypack changes are localized
- **WHEN** a developer needs to change keypack export/import handling
- **THEN** the change primarily occurs in the keypack submodule and does not require edits to encryption/decryption logic

### Requirement: Storage Secrets Refactor Preserves Behavior
The backend SHALL preserve existing secrets behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Storage Runs Repository Code Is Split Into Focused Submodules
The backend SHALL organize storage runs repository code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Event ingestion changes are localized
- **WHEN** a developer needs to change run event append/query logic
- **THEN** the change primarily occurs in the events submodule and does not require edits to run lifecycle or retention logic

### Requirement: Storage Runs Repository Refactor Preserves Behavior
The backend SHALL preserve existing runs repository behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Storage Notifications Repository Code Is Split Into Focused Submodules
The backend SHALL organize storage notification repository code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Enqueue changes are localized
- **WHEN** a developer needs to change enqueue selection or insertion logic
- **THEN** the change primarily occurs in the enqueue submodule and does not require edits to claiming or queue query logic

### Requirement: Storage Notifications Repository Refactor Preserves Behavior
The backend SHALL preserve existing notification repository behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Engine Notifications Code Is Split Into Focused Submodules
The backend SHALL organize engine notification code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Template changes are localized
- **WHEN** a developer needs to add or modify template placeholders
- **THEN** the change primarily occurs in the template submodule and does not require edits to sending or worker loop logic

### Requirement: Engine Notifications Refactor Preserves Behavior
The backend SHALL preserve existing notification behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: HTTP Notifications Module Is Split Into Focused Submodules
The backend SHALL organize HTTP notification-related handler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Queue handling is localized
- **WHEN** a developer needs to modify notification queue operations (list/cancel/retry)
- **THEN** the change primarily occurs in the queue submodule and does not require edits to settings or destination management

### Requirement: HTTP Notifications Refactor Preserves Behavior
The backend SHALL preserve existing HTTP notifications behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Scheduler Worker Is Split Into Focused Submodules
The backend SHALL organize scheduler worker code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Dispatch logic is localized
- **WHEN** a developer needs to modify agent dispatch behavior
- **THEN** the change primarily occurs in the dispatch submodule and does not require edits to local run execution or target storage code

### Requirement: Scheduler Worker Refactor Preserves Behavior
The backend SHALL preserve existing scheduler worker behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: HTTP Secrets Module Is Split Into Focused Submodules
The backend SHALL organize HTTP secret-related handler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Secret types are localized
- **WHEN** a developer needs to modify WebDAV secret handling
- **THEN** the change primarily occurs in the WebDAV submodule and does not require edits to SMTP or WeCom secret handlers

### Requirement: HTTP Secrets Refactor Preserves Behavior
The backend SHALL preserve existing HTTP secrets behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: HTTP Jobs Module Is Split Into Focused Submodules
The backend SHALL organize HTTP job-related handler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Validation logic is localized
- **WHEN** a developer needs to modify job spec validation rules
- **THEN** the change primarily occurs in the validation submodule and does not require edits to websocket or CRUD handlers

### Requirement: HTTP Jobs Refactor Preserves Behavior
The backend SHALL preserve existing HTTP jobs behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Agent Client Is Split Into Focused Submodules
The backend SHALL organize agent client code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Task handling is localized
- **WHEN** a developer needs to modify backup task handling
- **THEN** the change primarily occurs in the task-handling submodule and does not require edits to identity or websocket connection logic

### Requirement: Agent Client Refactor Preserves Behavior
The backend SHALL preserve existing agent client behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Filesystem Tar Writer Is Split Into Focused Submodules
The backend SHALL organize filesystem tar writing code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Hardlink handling is localized
- **WHEN** a developer needs to modify hardlink behavior
- **THEN** the change primarily occurs in the entry-writing submodule and does not require edits to encryption/part orchestration

### Requirement: Filesystem Tar Refactor Preserves Behavior
The backend SHALL preserve existing filesystem tar output behavior while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Agent Offline Module Is Split Into Focused Submodules
The backend SHALL organize agent offline scheduling/sync code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Cron matching logic is localized
- **WHEN** a developer needs to modify cron parsing/normalization
- **THEN** the change primarily occurs in the cron submodule and does not require edits to offline run persistence

### Requirement: Agent Offline Refactor Preserves Behavior
The backend SHALL preserve existing agent offline behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: HTTP Agents Module Is Split Into Focused Submodules
The backend SHALL organize HTTP agent-related handler code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Websocket protocol handling is localized
- **WHEN** a developer needs to modify agent websocket message handling
- **THEN** the change primarily occurs in the websocket submodule and does not require edits to admin CRUD handlers

### Requirement: HTTP Agents Refactor Preserves Behavior
The backend SHALL preserve existing HTTP agents behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Filesystem Backup Module Is Split Into Focused Submodules
The backend SHALL organize filesystem backup implementation code into focused Rust modules to improve maintainability, without changing behavior.

#### Scenario: Filesystem backup code navigation is localized
- **WHEN** a developer needs to change entries index serialization
- **THEN** the change primarily occurs in the entries index module and does not require edits to tar writing logic

### Requirement: Filesystem Backup Refactor Preserves Behavior
The backend SHALL preserve existing filesystem backup behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Scheduler Module Is Split Into Focused Submodules
The backend SHALL organize scheduler implementation code into focused Rust modules to improve maintainability, without changing scheduler behavior.

#### Scenario: Scheduler code navigation is localized
- **WHEN** a developer needs to change cron normalization behavior
- **THEN** the change primarily occurs in the scheduler cron module and does not require edits to queue/orchestration logic

### Requirement: Scheduler Refactor Preserves Behavior
The backend SHALL preserve existing scheduler behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing scheduler tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Restore Module Is Split Into Focused Submodules
The backend SHALL organize backup restore implementation code into focused Rust modules (entries index listing, access resolution, operation orchestration, unpacking, and verification) to improve maintainability.

#### Scenario: Restore code navigation is localized
- **WHEN** a developer needs to change restore entries listing behavior
- **THEN** the change primarily occurs in the entries index module and does not require edits to unpacking logic

### Requirement: Restore Refactor Preserves Behavior
The backend SHALL preserve existing restore/verify behavior and API contracts while refactoring internal module structure.

#### Scenario: Existing tests still pass
- **WHEN** `cargo test --workspace` is executed
- **THEN** all existing tests pass

### Requirement: Fallback Error Classification Avoids `internal_error` for Common Root Causes
When a request fails due to a common, user-actionable root cause, the backend SHALL classify it into a stable 4xx/403 error code rather than returning HTTP 500 `internal_error`.

The fallback classification SHALL cover at least:
- IO permission denied -> HTTP 403 `permission_denied`
- IO path not found -> HTTP 404 `path_not_found`
- DB row not found -> HTTP 404 `not_found`

#### Scenario: Permission denied is returned as 403 with a stable code
- **WHEN** a request fails due to `std::io::ErrorKind::PermissionDenied`
- **THEN** the backend responds with HTTP 403
- **AND** the response includes `error = "permission_denied"`

#### Scenario: Not found is returned as 404 with a stable code
- **WHEN** a request fails due to `std::io::ErrorKind::NotFound`
- **THEN** the backend responds with HTTP 404
- **AND** the response includes `error = "path_not_found"`

#### Scenario: DB row not found is returned as 404 with a stable code
- **WHEN** a request fails due to `sqlx::Error::RowNotFound`
- **THEN** the backend responds with HTTP 404
- **AND** the response includes `error = "not_found"`

### Requirement: Debug Error Details Are Off by Default
The backend SHALL support an opt-in “debug error details” mode for troubleshooting.

When debug error details are disabled (default), the backend SHALL NOT include internal diagnostic information in responses.

When enabled, the backend MAY include safe diagnostics in `details.debug` for HTTP 500 `internal_error` responses only.

#### Scenario: Default mode does not expose internal diagnostics
- **WHEN** debug error details are disabled
- **AND** a request fails and results in HTTP 500 `internal_error`
- **THEN** the response does not include `details.debug`

#### Scenario: Debug mode includes internal diagnostics for internal_error only
- **WHEN** debug error details are enabled
- **AND** a request fails and results in HTTP 500 `internal_error`
- **THEN** the response MAY include `details.debug`

### Requirement: Agent Run Ingest Enforces Size Limits
The backend SHALL enforce request body size limits for HTTP endpoints, including the Agent run ingest endpoint.

#### Scenario: Oversized ingest request is rejected
- **WHEN** an Agent sends a run ingest request exceeding the configured maximum size
- **THEN** the backend rejects the request with an appropriate HTTP error status (e.g., 413)

### Requirement: Agent Run Ingest Validates Payload and Is Idempotent
The backend SHALL validate Agent run ingest payloads and SHALL ingest in an idempotent manner.

#### Scenario: Ingest validates timestamps and required fields
- **WHEN** an Agent ingests a run record with invalid timestamps or missing required fields
- **THEN** the backend responds with a 4xx error and a stable error code

#### Scenario: Re-ingesting the same run does not create duplicates
- **WHEN** an Agent ingests the same run ID multiple times
- **THEN** the backend does not create duplicate run events for the same `(run_id, seq)`

### Requirement: Ingest Can Upsert Run Metadata
The backend SHALL support upserting run metadata (status/ended_at/summary/error) for an existing run ID during Agent ingest.

#### Scenario: Ingest updates an existing run record
- **WHEN** an Agent ingests a run ID that already exists in the database
- **THEN** the backend updates the run’s metadata to reflect the ingested payload

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

### Requirement: Backend Is Split Into Focused Crates
The backend codebase SHALL be organized into focused crates with clear responsibilities (HTTP, engine/orchestration, storage, backup, targets, notifications, and shared core types).

#### Scenario: Adding a target is isolated
- **WHEN** a new backup target type (e.g., S3) is implemented
- **THEN** the implementation primarily lives in the targets crate and does not require refactoring unrelated HTTP routing code

### Requirement: Backend Crate Dependencies Are Layered
The backend crate dependency graph SHALL be acyclic and SHOULD follow a layered architecture where the HTTP crate depends on the engine layer rather than directly depending on storage/backup/targets internals.

#### Scenario: HTTP does not bypass the engine layer
- **WHEN** an HTTP handler triggers a backup run or restore action
- **THEN** the handler calls into the engine layer rather than reaching into low-level backup/target modules directly

### Requirement: Core Types Remain Lightweight
The shared core crate (`bastion-core`) SHALL remain lightweight and MUST NOT depend on heavy runtime frameworks such as `axum`, `tokio`, `reqwest`, or `sqlx`.

#### Scenario: Core crate stays framework-free
- **WHEN** `bastion-core` is compiled
- **THEN** it does not pull in HTTP/runtime/storage framework dependencies

### Requirement: Standard Error Response Includes Optional Details
The backend SHALL return errors using a standard JSON body containing:
- `error` (machine-readable error code)
- `message` (human-readable message)
- `details` (optional structured diagnostics)

`details` SHALL NOT include secrets or sensitive values.

#### Scenario: Backend returns a structured error with details
- **WHEN** a request fails due to invalid user input
- **THEN** the backend responds with a 4xx status
- **AND** the response JSON includes `error` and `message`
- **AND** the response JSON includes `details` with safe structured fields (e.g. `field`)

### Requirement: Validation Failures Use 4xx and Stable Error Codes
The backend SHALL surface user-input validation failures as 4xx responses with stable `error` codes and actionable messages, rather than returning HTTP 500.

#### Scenario: Invalid WeCom webhook URL returns 400 with field details
- **WHEN** the user saves a WeCom bot secret with a webhook URL that cannot be parsed as a URL
- **THEN** the backend responds with HTTP 400
- **AND** the response includes `error = "invalid_webhook_url"`
- **AND** `details.field = "webhook_url"`

#### Scenario: Invalid SMTP mailbox returns 400 with field details
- **WHEN** the user saves an SMTP secret with an invalid `from` email address
- **THEN** the backend responds with HTTP 400
- **AND** the response includes `error = "invalid_from"`
- **AND** `details.field = "from"`

### Requirement: Rate Limit Responses Include Retry-After Details
When the backend rate-limits login attempts, it SHALL include machine-readable retry information in `details`.

#### Scenario: Login rate limit includes retry-after seconds
- **WHEN** a client is rate-limited during login
- **THEN** the backend responds with HTTP 429
- **AND** the response includes `error = "rate_limited"`
- **AND** `details.retry_after_seconds` is present

### Requirement: Backend Clippy Warnings Are Zero
The backend SHALL pass clippy with warnings treated as errors.

#### Scenario: Developer runs strict clippy locally
- **WHEN** the developer runs `cargo clippy --all-targets --all-features -- -D warnings`
- **THEN** the command exits successfully without clippy warnings

### Requirement: CI Rejects Clippy Warnings
The project CI scripts SHALL treat clippy warnings as errors to prevent new warnings from being introduced.

#### Scenario: CI runs strict clippy
- **WHEN** CI runs the backend clippy step
- **THEN** clippy warnings cause the job to fail

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

### Requirement: Raw-Tree Format Disables Encryption
When a job is configured to use the `raw_tree_v1` artifact format, the backend SHALL reject payload encryption settings that require tar-based packaging.

#### Scenario: Raw-tree with encryption is rejected
- **WHEN** a user configures a job with artifact format `raw_tree_v1` and enables age encryption
- **THEN** the backend rejects the configuration with a clear validation error

### Requirement: Backend SHALL emit canonical envelopes for remaining bridged and execute-stage failures
Backend failure diagnostics that are bridged from Agent messages or produced during execute-stage processing SHALL include canonical `error_envelope` fields in event payloads.

#### Scenario: Agent snapshot-delete failure is bridged with canonical envelope
- **GIVEN** Hub receives a snapshot delete failure/result from an Agent
- **WHEN** backend appends task/run events for that failure
- **THEN** event fields SHALL include `error_envelope` with stable `schema_version`, `code`, `kind`, `retriable`, and `transport.protocol`
- **AND** legacy fields SHALL remain present during migration

#### Scenario: Execute-stage failure/warn event includes canonical envelope
- **GIVEN** an execute-stage path (filesystem/sqlite/vaultwarden) emits a warning or failure event
- **WHEN** backend appends the event
- **THEN** fields SHALL include `error_envelope` with stable code namespace and origin metadata
- **AND** transport metadata SHALL reflect the best-known protocol context when available

### Requirement: Backend SHALL synthesize envelopes when upstream payload lacks canonical diagnostics
When upstream/bridged payloads do not include canonical envelope fields, backend SHALL synthesize a valid envelope from available structured context.

#### Scenario: Agent result lacks envelope but has status/error_kind
- **GIVEN** a bridged Agent result includes legacy status/error fields but no envelope
- **WHEN** backend persists the corresponding event
- **THEN** backend SHALL synthesize `error_envelope` using stable fallback mapping
- **AND** synthesized envelope SHALL preserve retriable and error-kind semantics when inferable

### Requirement: Rollout SHALL remain backward-compatible for existing clients
Migration in this phase SHALL preserve existing readers that still consume legacy fields.

#### Scenario: Legacy event readers continue to function
- **GIVEN** a client that only reads legacy diagnostic fields
- **WHEN** backend emits the new envelope fields
- **THEN** legacy fields SHALL remain available with unchanged compatibility behavior

### Requirement: Runtime Log Filter Resolution Is Explicit-Input Driven
Backend runtime log filter resolution SHALL derive `RUST_LOG` fallback behavior from explicit caller-provided input rather than reading mutable process environment from resolution helpers.

#### Scenario: Runtime entry point supplies captured RUST_LOG
- **GIVEN** a backend runtime entry point needs to resolve effective logging configuration
- **WHEN** `RUST_LOG` is set in the process environment
- **THEN** the entry point captures that value and passes it into runtime-config resolution explicitly
- **AND** downstream resolution helpers do not read `RUST_LOG` directly

### Requirement: Runtime Config Resolution Tests Are Parallel-Safe
Backend unit tests covering runtime config log-filter precedence SHALL remain deterministic when the Rust test runner executes tests in parallel.

#### Scenario: DB fallback test runs alongside env-precedence test
- **GIVEN** one test validates `RUST_LOG` precedence and another validates DB fallback behavior
- **WHEN** the backend test binary runs with parallel test threads
- **THEN** each test controls its own log-filter inputs explicitly
- **AND** the DB fallback assertion does not observe leaked process-global `RUST_LOG` state
