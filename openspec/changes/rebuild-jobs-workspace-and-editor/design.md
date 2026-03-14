## Context

Jobs is Bastion's central workflow, but the current design splits the operator's attention:

- the list must support multiple modes and layouts at once
- split mode hides important filters behind a small affordance
- detail content is partly workspace-like, but editing still depends on a large modal
- mobile adapts the same concepts, but it still inherits the structure of the desktop implementation rather than a dedicated task flow

This change turns Jobs into a true workspace with clear responsibilities:

- list and saved views for discovery
- detail/inspection for understanding
- stepper authoring for create/edit

## Goals / Non-Goals

**Goals:**
- make Jobs the product's primary operational workspace
- improve filter discoverability and make common views reusable
- replace modal authoring with a full-page stepper
- expose clearer operational summary fields for each job
- keep desktop and mobile behavior aligned conceptually without forcing identical layouts

**Non-Goals:**
- redesigning run inspection in depth (handled by the Runs workspace change)
- changing the semantics of job schedules, sources, targets, or retention logic
- introducing an entirely new backup-job data model on the backend

## Decisions

### 1. Desktop Jobs will use a three-pane workspace

The desktop route will render:

- a list/discovery pane
- a primary job detail pane
- a secondary inspection/action pane for recent runs, health, or related summaries

Rationale:
- operators need to keep list context while evaluating a selected job
- the current split/list toggle still behaves like a page that is trying to be both a table and a workspace

Alternatives considered:
- keep the current two-pane split and only make filters larger
  - rejected because it would still leave details and supporting action context competing for the same space
- move detail into a separate page only
  - rejected for desktop because it would degrade list-to-detail scanning efficiency

#### Workspace route model

The canonical Jobs route family will be:

- `/jobs`
- `/jobs/new`
- `/jobs/:jobId`
- `/jobs/:jobId/overview`
- `/jobs/:jobId/history`
- `/jobs/:jobId/data`
- `/jobs/:jobId/edit`

Collection routes accept query state such as:

- `scope=<all|hub|agent:ID>`
- `view=<savedViewId>`
- `q=...`
- `latest_status=...`
- `schedule_mode=...`
- `include_archived=...`
- `sort=...`
- `page=...`
- `page_size=...`

Detail routes may additionally preserve list return context through query keys such as:

- `from_scope`
- `from_view`
- `from_page`

The stable path always identifies the job object itself; query state is for list context restoration and secondary panes, not for object identity.

#### Responsive pane behavior

The workspace uses one conceptual model with width-based collapse:

- `>= 1440px`: list pane, primary detail pane, and supporting pane may render simultaneously
- `1024px - 1439px`: list pane and primary detail pane remain visible; the supporting pane collapses into tabs or a secondary drawer
- `768px - 1023px`: the page may use a compact split/list-detail arrangement, but it must not require a permanently visible third pane
- `< 768px`: the Jobs surface uses dedicated list and detail pages rather than a split workspace

### 2. Filters will be persistent and reusable

The workspace will expose always-visible primary filters on desktop and a clear filter drawer on mobile. Reusable saved views will capture common list states such as:

- failed recently
- stale or never-verified jobs
- archived jobs
- node-specific scopes

Rationale:
- jobs scale quickly, and ad-hoc filters are not enough once operators build habits around recurring slices

Alternatives considered:
- keep transient filters only
  - rejected because they reset too much cognitive work for frequent operators

#### Saved-view schema

V1 saved views will be browser-persisted rather than server-synced. This keeps the first implementation tractable while still supporting repeatable operator workflows.

Illustrative schema:

```json
{
  "id": "jobs-failed-recently",
  "name": "Failed recently",
  "scope": "all",
  "filters": {
    "q": "",
    "latest_status": "failed",
    "schedule_mode": "all",
    "include_archived": false,
    "sort": "updated_desc"
  },
  "created_at": 1760000000,
  "updated_at": 1760000300
}
```

Contract rules:

- saved views are keyed per authenticated user/browser profile
- the schema is versioned so filters can evolve without corrupting stored state
- built-in starter views such as `Failed recently` may be shipped as non-destructive defaults, but custom saved views remain operator-owned
- applying a saved view updates collection query state in a single transition instead of mutating filters one field at a time

### 3. Job editing will move to a full-page stepper

Create and edit flows will use the same full-page step structure:

- `Basic`
- `Source`
- `Target`
- `Schedule & Retention`
- `Security`
- `Notifications`
- `Review`

Rationale:
- the current modal is too dense for a central, configuration-heavy workflow
- the user needs room for explanation, validation, and summary without modal constraints

Alternatives considered:
- keep the existing modal and refactor internals only
  - rejected because the modal container remains the main UX limitation

#### Editor route behavior

Create and edit routes are explicit:

- `/jobs/new`
- `/jobs/:jobId/edit`

The editor owns its own draft state and does not share selection state with the list workspace. Entering the editor suspends list-pane focus, but returning from save/cancel can restore the originating list context through `from_*` query metadata when present.

### 4. The editor will use progressive validation with draft persistence

Each step will validate locally and against server rules as needed. The flow will preserve in-progress drafts so users can navigate away and resume.

Rationale:
- backup jobs often require external details (paths, credentials, labels, schedules) and are easy to interrupt

Alternatives considered:
- validate only on final submit
  - rejected because late validation makes complex forms frustrating and opaque

#### Draft-state contract

V1 draft persistence will be client-persisted and keyed separately for create and edit flows.

Illustrative draft envelope:

```json
{
  "version": 1,
  "mode": "edit",
  "job_id": "job_123",
  "base_job_updated_at": 1759990000,
  "last_step": "target",
  "values": {},
  "updated_at": 1760000000
}
```

Rules:

- create drafts are keyed independently from edit drafts so an unfinished new job cannot overwrite an edit-in-progress
- edit drafts store `base_job_updated_at` to detect stale edits when the underlying job changed elsewhere
- the editor autosaves on step transitions and debounced field changes, but only stores serializable authoring state
- successful submit or explicit discard removes the corresponding draft
- when a stale edit draft is resumed, the page must offer resume, discard, or reload-from-live behavior before allowing a blind overwrite

### 5. Job detail pages will consume a workspace-oriented read model

The UI will not build job summaries by stitching together many unrelated calls. The system will provide a job workspace view model containing:

- job identity and summary
- last success/failure
- next schedule
- target and restore-readiness summaries
- related integrations and warnings

Rationale:
- a workspace page needs stable, explicit summary contracts

#### Job workspace read-model contract

The backend will expose workspace-oriented list and detail view models rather than forcing the client to combine the current list response, job detail response, latest runs response, retention response, snapshot response, and integration lookups ad hoc.

Illustrative collection response:

```json
{
  "scope": { "requested": "all", "effective": "all" },
  "filters": {
    "q": "",
    "latest_status": "failed",
    "schedule_mode": "all",
    "include_archived": false,
    "sort": "updated_desc"
  },
  "page": 1,
  "page_size": 20,
  "total": 3,
  "items": [
    {
      "id": "job_123",
      "name": "Nightly DB",
      "scope": "agent:db-a",
      "latest_run": { "id": "run_1", "status": "failed", "started_at": 1759990000, "ended_at": 1759990300 },
      "next_run_at": 1760010000,
      "health": "critical",
      "readiness": { "last_success_at": 1759900000, "last_verify_at": null },
      "warnings": ["verify_missing"],
      "capabilities": { "can_run_now": true, "can_edit": true }
    }
  ]
}
```

Illustrative detail response:

```json
{
  "job": {
    "id": "job_123",
    "name": "Nightly DB",
    "scope": "agent:db-a"
  },
  "summary": {
    "latest_success_at": 1759900000,
    "latest_failure_at": 1759990300,
    "next_run_at": 1760010000
  },
  "readiness": {
    "last_restorable_run_id": "run_0",
    "last_verify_at": null,
    "state": "warning"
  },
  "recent_runs": [],
  "related_integrations": [],
  "warnings": [],
  "capabilities": {
    "can_run_now": true,
    "can_edit": true,
    "can_archive": true,
    "can_delete": true
  }
}
```

This allows the list pane, detail pane, and supporting pane to share one consistent definition of job health and readiness.

## Risks / Trade-offs

- [Three-pane desktop layout could feel heavy on narrower laptops] → use responsive collapse thresholds and allow the third pane to fold into tabs or drawers below a width breakpoint
- [Saved views increase state complexity] → define a small, explicit saved-view schema and phase it behind a minimal first implementation
- [Full-page editor increases route/state complexity] → keep create/edit routes explicit and isolate editor draft state from list-selection state
- [Workspace read model may require denormalized queries] → define the API contract first, then optimize query composition incrementally

## Migration Plan

1. Add new Jobs routes and layout containers alongside current implementations.
2. Introduce the new job workspace read model and use it in the new detail pane first.
3. Add the full-page editor flow and route create/edit actions to it.
4. Migrate desktop and mobile list/detail navigation to the new workspace.
5. Remove the modal editor only after parity is reached for create/edit flows.

Rollback:
- keep the modal editor implementation until the new full-page flow is validated
- preserve current list mode as a compatibility fallback during the workspace rollout

#### Temporary alias mapping

Old node-scoped Jobs routes may be normalized during migration as follows:

- `/n/:nodeId/jobs` -> `/jobs?scope=<mapped>`
- `/n/:nodeId/jobs/:jobId` -> `/jobs/:jobId/overview?from_scope=<mapped>`
- `/n/:nodeId/jobs/:jobId/overview` -> `/jobs/:jobId/overview?from_scope=<mapped>`
- `/n/:nodeId/jobs/:jobId/history` -> `/jobs/:jobId/history?from_scope=<mapped>`
- `/n/:nodeId/jobs/:jobId/data` -> `/jobs/:jobId/data?from_scope=<mapped>`

These aliases exist only while internal links/tests are being updated to canonical routes. `<mapped>` resolves to `hub` for the hub node and `agent:<nodeId>` for agent-scoped old URLs.

## Open Questions

- Should saved views ship in the first implementation slice or immediately after the new persistent filters land?
- Which job health/readiness fields need to be materialized by the backend versus derived on demand in the UI?
