## Context

The current product centers navigation around jobs, while actual operator troubleshooting often centers around runs:

- a failed run is the natural starting point for diagnosis
- run detail content is dense and partly modal
- event streams require too much scanning before the root cause is obvious
- mobile inspection suffers the most because dense run detail content is stacked into a constrained dialog-style flow

This change makes Runs a first-class workspace and formalizes the diagnostics model so operators see concise failure explanations before raw payloads.

## Goals / Non-Goals

**Goals:**
- make `Runs` directly reachable as a primary product surface
- provide a dedicated run detail page for desktop and mobile
- move root-cause diagnostics to the top of the run inspection experience
- make event browsing scalable via server-driven filtering/pagination contracts
- expose restore/verify/cancel actions from the run context

**Non-Goals:**
- changing the underlying meaning of run status, operation status, or restore/verify behavior
- replacing the canonical event model with a brand-new observability subsystem
- redesigning the full Jobs workspace (handled by a separate change)

## Decisions

### 1. Runs will have a global top-level index

Operators will be able to browse runs independently of Jobs, using filters such as:

- status
- node/scope
- job
- run type or operation type
- time range

Rationale:
- many troubleshooting sessions begin with a failure or restore/verify operation, not with the job list

Alternatives considered:
- keep runs only under job history
  - rejected because it makes cross-job triage and failure-driven workflows slower

#### Runs route family

The canonical Runs route family will be:

- `/runs`
- `/runs/:runId`

Collection query state may include:

- `scope=<all|hub|agent:ID>`
- `status=...`
- `job_id=...`
- `kind=backup|restore|verify|cleanup`
- `range=<preset>` or `from`/`to`
- `q=...`
- `page=...`
- `page_size=...`

Detail routes may additionally preserve origin context with:

- `from_scope`
- `from_job`
- `from_view`

The path identifies the run object. Query state restores the collection slice the operator came from; it does not redefine the run's identity or owning scope.

### 2. Run detail will use a dedicated page rather than modal-first inspection

The run detail route will become a stable page that supports:

- summary and next-step actions
- diagnostic/event console
- related operations/artifacts/tabs

Rationale:
- run inspection is too important and too content-heavy for modal-first UX

Alternatives considered:
- keep current modal and only reorder content
  - rejected because route identity, mobile usability, and event-console density all still suffer

#### Run workspace structure

The dedicated run page will use three conceptual regions:

- summary rail: status, root cause, duration, scope, and primary actions
- diagnostics workspace: event console, first-error jump, and structured failure summary
- related context: operations, artifacts, restore/verify history, and owning job links

Responsive rules:

- desktop may render summary and diagnostics side by side
- compact desktop collapses related context into tabs under diagnostics
- mobile renders summary-first, then event console, then secondary context
- no primary diagnostic action may require reopening a modal from inside another modal

### 3. The run view model will surface structured root-cause data

The system will provide normalized fields such as:

- `failure_kind`
- `failure_stage`
- `failure_title`
- `failure_hint`
- `first_error_event_seq`

Rationale:
- operators should not need to infer the root cause by reading raw JSON or the last event line

Alternatives considered:
- continue deriving root cause purely in the UI from events
  - rejected because that is brittle, duplicated, and hard to keep consistent

#### Run detail read-model contract

Illustrative dedicated run view model:

```json
{
  "run": {
    "id": "run_123",
    "job_id": "job_456",
    "job_name": "Nightly DB",
    "scope": "agent:db-a",
    "status": "failed",
    "kind": "backup",
    "started_at": 1759990000,
    "ended_at": 1759990300
  },
  "diagnostics": {
    "state": "structured",
    "failure_kind": "transport",
    "failure_stage": "upload",
    "failure_title": "WebDAV upload failed",
    "failure_hint": "Retry later or lower direct-upload concurrency",
    "first_error_event_seq": 143,
    "root_cause_event_seq": 143
  },
  "capabilities": {
    "can_cancel": false,
    "can_restore": false,
    "can_verify": false
  },
  "related": {
    "operations": [],
    "artifacts": []
  }
}
```

Fallback rules:

- older or partially normalized runs may return `diagnostics.state = fallback`
- fallback mode must still provide enough summary text for a useful first screen
- raw event payload access remains mandatory even when structured diagnostics are present

### 4. Event browsing will be server-filterable and page-scalable

The event console will use server-side support for:

- query filtering
- level/kind filtering
- cursor/pagination
- locating the first error quickly

Rationale:
- long event histories should not require fetching and filtering the entire stream client-side

Alternatives considered:
- keep the current all-events-in-memory model
  - rejected because it does not scale and makes first-error navigation unnecessarily expensive

#### Event-console contract

The event console will use a windowed, sequence-ordered contract. V1 will extend the existing run-events API shape instead of requiring a brand-new observability subsystem.

Illustrative request parameters:

- `q`
- `levels=error,warn`
- `kinds=upload_failed,verify_started`
- `limit=100`
- `before_seq=220`
- `after_seq=120`
- `anchor=tail|first_error|seq:143`

Illustrative response shape:

```json
{
  "filters": {
    "q": "webdav",
    "levels": ["error"],
    "kinds": []
  },
  "window": {
    "first_seq": 143,
    "last_seq": 220,
    "has_older": true,
    "has_newer": false
  },
  "locators": {
    "first_error_seq": 143,
    "root_cause_seq": 143
  },
  "items": []
}
```

Contract rules:

- items are always returned in ascending `seq` order
- filters are applied before windowing so paging remains stable for the chosen slice
- without an explicit cursor or anchor, the endpoint returns the latest matching window rather than the entire event history
- `anchor=first_error` centers or starts the window at the first known error location when available
- websocket/live transport reuses the same filter semantics and resumes from `after_seq=<last_rendered_seq>`

### 5. Mobile run detail will use a task-first order

On mobile, the first screen will prioritize:

- status
- root cause
- stage
- duration
- immediate next actions

The event console moves below that task-first summary.

Rationale:
- mobile operators need the answer first, not the full detail structure first

## Risks / Trade-offs

- [A new top-level Runs surface duplicates some job-history affordances] → treat Job history as a contextual slice of Runs rather than a separate source of truth
- [Structured diagnostics may lag behind rare failure modes] → preserve raw event payload access and allow fallback rendering when normalization is incomplete
- [Server-side event filtering adds API and storage query complexity] → keep the contract narrow and use cursor/keyset patterns where possible
- [Deep-link migrations from current job-scoped run paths may be fragile] → add minimal client-side alias coverage and route tests early

## Migration Plan

1. Add top-level Runs routes and only the minimal temporary aliases needed while current job-scoped internal links are being removed.
2. Implement the structured run view model and dedicated page layout.
3. Add the server-driven event console API and switch the new run page to use it.
4. Update command-center/jobs links to open the dedicated run detail route.
5. De-emphasize modal run inspection once parity is reached.

Rollback:
- preserve the current run-detail modal path during rollout
- keep the old events-loading path available until the new event contract is verified

#### Temporary alias mapping

Old job-scoped run entry points may be normalized during migration as follows:

- `/n/:nodeId/jobs/:jobId/:section/runs/:runId` -> `/runs/:runId?from_job=<jobId>&from_scope=<mapped>`
- job-history drawers or dialogs become canonical `/runs/:runId` links once parity is reached

`<mapped>` resolves to `hub` for hub-owned runs and `agent:<nodeId>` for agent-scoped legacy URLs.

#### Live-refresh recovery

Active run pages must keep the current diagnostic window stable during live refresh:

- initial load fetches a bounded HTTP event window
- live updates continue from the last rendered sequence
- transport failure surfaces explicit connection state instead of silently freezing
- reconnect resumes from the last stable sequence and preserves current filters when possible
- if live transport cannot be resumed, the page falls back to explicit refresh or bounded polling without dropping existing event context

## Open Questions

- Should the first version of the Runs index include restore/verify operations inline, or should those remain visible only from run detail until a later iteration?
- Which diagnostics fields can be guaranteed for all historical runs versus only newly generated runs?
