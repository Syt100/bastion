## Context

Bastion's current UI shell grew around implementation boundaries:

- the "Dashboard" is an overview page, but it does not function as a true operational command center
- `Jobs` is the only partially workspace-like area, while `Runs` remains subordinate and difficult to inspect quickly
- node scope is encoded in primary paths such as `/n/:nodeId/...`, which is easy for implementation but heavy for operators to reason about
- the visual system favors repeated elevated cards, which flattens hierarchy and makes high-priority information compete with secondary metadata

This change establishes the control-console foundation that later changes will build on. It intentionally focuses on shell, navigation, route model, and landing-page behavior before deeper surface rebuilds.

## Goals / Non-Goals

**Goals:**
- define a durable top-level information architecture that matches operator workflows
- make the landing page attention-oriented rather than metric-oriented
- separate "object identity" from "execution scope" in route design for primary surfaces
- codify a panel-and-rail console hierarchy in the design system so later workspace pages are visually coherent
- preserve mobile usability and provide a short migration path from existing internal links/history

**Non-Goals:**
- implementing the full Jobs, Runs, Fleet, Integrations, or System rebuilds in this change
- redesigning every secondary settings page
- changing backup, scheduling, or agent semantics
- removing all old internal entry points immediately without minimal alias coverage

## Decisions

### 1. Primary navigation will be organized by operator task domains

The shell will expose six primary surfaces:

- `Command Center`
- `Jobs`
- `Runs`
- `Fleet`
- `Integrations`
- `System`

Rationale:
- these labels map directly to the operator's mental model and the product plan
- they separate daily operations from low-frequency administration
- they create stable homes for future pages without overloading `Settings`

Alternatives considered:
- keep the existing Dashboard / Jobs / Agents / Settings model and only polish pages
  - rejected because the current grouping is the root cause of the product feeling fragmented
- move everything under Jobs and keep Runs/Fleet secondary
  - rejected because runs and fleet health are first-class operational concerns, not subordinate detail tabs

### 2. Primary object routes will be stable, while scope becomes explicit state

Primary surfaces and objects will use stable top-level routes such as:

- `/`
- `/jobs`
- `/jobs/:jobId`
- `/runs`
- `/runs/:runId`
- `/fleet`
- `/fleet/:agentId`
- `/integrations/...`
- `/system/...`

Node scope will be represented via explicit selector state and route/query context for pages that need it.

Rationale:
- stable links are easier to understand, share, and preserve over time
- route identity should represent the object or surface being viewed, not the current implementation shape
- explicit scope chips/selectors are easier to read than leading `n/:nodeId` prefixes

Alternatives considered:
- keep `/n/:nodeId/...` as the primary route model
  - rejected because it leaks implementation details and makes global versus scoped navigation harder to understand
- remove scope entirely and always show global state
  - rejected because operators still need to focus on hub-only or node-specific execution context

#### Canonical route families

The shell will treat the following paths as canonical operator-facing routes:

| Area | Canonical routes | Scope behavior |
| --- | --- | --- |
| Command Center | `/` | supports `?scope=<all|hub|agent:ID>` and `?range=<preset>` |
| Jobs | `/jobs`, `/jobs/new`, `/jobs/:jobId`, `/jobs/:jobId/:section`, `/jobs/:jobId/edit` | collection routes support explicit scope and saved-view context; detail routes resolve object identity first |
| Runs | `/runs`, `/runs/:runId` | collection routes support explicit scope; detail routes keep object identity stable and may carry return-context query state |
| Fleet | `/fleet`, `/fleet/:agentId` | collection routes default to `all`; detail routes are object-scoped |
| Integrations | `/integrations`, `/integrations/storage`, `/integrations/notifications`, `/integrations/distribution` | scope-aware subsections use query scope instead of node-prefixed paths |
| System | `/system`, `/system/runtime`, `/system/maintenance`, `/system/appearance`, `/system/about` | not scope-aware |

Canonical scope strings will be:

- `all`
- `hub`
- `agent:<agentId>`

The shell will persist `preferredScope` in client UI state. Day-one default is `all`; if the product only has the hub, the UI may render that state as a visually simplified hub-only selection without changing the underlying contract.

#### Scope precedence and normalization

The routing model distinguishes three concepts:

- `preferredScope`: persisted shell preference shared across collection pages
- `explicitScope`: query-derived scope carried by the current route
- `objectScope`: scope resolved by the backend for an object detail page such as a job, run, or fleet member

Resolution rules:

1. Collection pages (`/`, `/jobs`, `/runs`, `/fleet`, scope-aware integrations pages) use `explicitScope ?? preferredScope ?? all`.
2. Detail pages fetch by stable object identity first; `objectScope` governs the object's own data even when `explicitScope` is present.
3. On detail pages, `explicitScope` is retained only as contextual state for back-navigation, related-list slices, or secondary panes.
4. If a detail-page `explicitScope` excludes the object's own scope, the page normalizes object panels to `objectScope`, preserves the original context for return links, and surfaces that normalization in visible UI state.
5. Changing scope from the shell updates `preferredScope` only. It does not mutate object-specific URLs or overwrite explicit route query state already present in the current page.

This avoids ambiguous cases such as a stable `/runs/:runId` page pretending that a run belongs to a different node just because the operator previously selected another scope in the shell.

### 3. Command Center will use an aggregated attention model instead of raw KPI emphasis

The landing page will be built around:

- `Needs Attention`
- `Recent Critical Activity`
- `Recovery Readiness`
- `Upcoming / Watchlist`

The backend/API contract will return already-grouped, scope-aware sections rather than requiring the UI to infer attention from many unrelated counters.

Rationale:
- the landing page should answer "what needs action?" before "what are the totals?"
- aggregation logic is easier to validate and keep consistent on the server side

Alternatives considered:
- keep the current dashboard API and restyle the cards
  - rejected because the issue is product framing, not only layout

#### Command Center read-model contract

The Command Center will use a dedicated aggregated endpoint instead of stitching together dashboard, jobs, runs, agents, and notification slices on the client. The view model will be designed so each section can independently be `ready`, `empty`, or `degraded`.

Illustrative response shape:

```json
{
  "generated_at": 1760000000,
  "scope": {
    "requested": "agent:edge-a",
    "effective": "agent:edge-a"
  },
  "range": {
    "preset": "24h",
    "from": 1759913600,
    "to": 1760000000
  },
  "attention": {
    "state": "ready",
    "items": [
      {
        "id": "run:run_123",
        "kind": "run_failed",
        "severity": "critical",
        "title": "Nightly backup failed",
        "summary": "WebDAV upload returned 429",
        "occurred_at": 1759990000,
        "scope": "agent:edge-a",
        "primary_action": { "label": "Open run", "href": "/runs/run_123" },
        "secondary_action": { "label": "Open job", "href": "/jobs/job_456" }
      }
    ]
  },
  "critical_activity": {
    "state": "ready",
    "items": []
  },
  "recovery_readiness": {
    "state": "degraded",
    "overall": "degraded",
    "backup": { "recent_success_at": 1759980000, "stale_jobs": 2 },
    "verify": { "recent_success_at": null, "missing_jobs": 4 },
    "blockers": []
  },
  "watchlist": {
    "state": "empty",
    "items": []
  }
}
```

Contract rules:

- section failure must not fail the whole payload when the rest of the read model can still be produced
- action targets must always use canonical stable routes
- item ids must be stable enough for list rendering, deep linking, and optimistic refresh
- scope and range echo fields must be returned on every response so the client can prove which aggregate was rendered

### 4. The design system will prefer panel/rail hierarchy over repeated elevated cards

The console shell and primary work surfaces will use:

- one dominant content surface per area
- side rails / inset panels for secondary controls
- explicit attention sections for risks
- more restrained shell chrome

Rationale:
- repeated cards flatten priority
- operators scan better when one surface owns the page and secondary information is visibly subordinate

Alternatives considered:
- retain the current card-heavy language and tweak spacing only
  - rejected because it would preserve the current hierarchy problems

### 5. Migration will be phased and alias-backed

Legacy node-scoped URLs will not be treated as long-term supported public contracts. During rollout, the client router may temporarily normalize a small set of old internal paths into canonical routes while the app, tests, and docs are updated.

Rationale:
- existing deep links, docs, tests, and operator habits must keep working during the transition

Alternatives considered:
- immediate hard cutover
  - rejected because the shell/routing work is cross-cutting and will land incrementally

## Risks / Trade-offs

- [Scope semantics become ambiguous during migration] → define explicit precedence between route object identity, route/query scope, and global preferred scope before implementation begins
- [Command Center aggregation becomes too expensive] → use a dedicated read model/aggregation endpoint and allow phased server-side optimization without changing UI contracts
- [Partial rollout creates mixed old/new navigation] → keep temporary aliases and navigation metadata centralized in one shell module so old and new pages can coexist predictably for one migration window
- [Visual-system changes drift into page-specific one-offs] → encode shell/panel rules in shared design-system requirements and primitives before page rewrites

## Migration Plan

1. Add the new navigation metadata and a minimal set of client-side route aliases without removing current routes immediately.
2. Introduce the Command Center endpoint and page while old Dashboard entry points normalize there.
3. Migrate primary navigation to the new shell on desktop and mobile.
4. Update internal links, tests, and docs to the canonical top-level routes as downstream surfaces migrate.
5. Remove temporary aliases and obsolete navigation branches only after Jobs, Runs, Fleet, Integrations, and System have all landed on the new IA.

Rollback:
- retain temporary aliases and legacy navigation metadata behind a feature flag or route branch until the new shell is proven
- preserve the old dashboard page implementation until the Command Center endpoint and page are validated

#### Temporary alias matrix

Phase 1 will introduce canonical routes plus a minimal set of client-side aliases. These aliases exist only to smooth in-flight migration of browser history, internal links, and tests; they are expected to be removed after the transition.

| Old internal path | Canonical normalized target | Notes |
| --- | --- | --- |
| `/n/:nodeId` | `/jobs?scope=<mapped>` | only needed while node-root links still exist internally |
| `/n/:nodeId/jobs` | `/jobs?scope=<mapped>` | preserves collection scope |
| `/n/:nodeId/jobs/:jobId/:section/runs/:runId` | `/runs/:runId?from_job=<jobId>&from_scope=<mapped>` | canonical run detail is top-level |
| `/agents` | `/fleet` | remove after navigation/tests are updated |
| `/settings/...` | `/integrations/...` or `/system/...` | only keep aliases for routes still referenced during migration |

`<mapped>` uses `hub` for the hub node and `agent:<nodeId>` for agent-specific legacy paths.

#### Desktop/mobile shell boundaries

The shell must preserve one information architecture while adapting affordances by width:

- `< 768px`: drawer-first shell, sticky top bar, one contextual secondary-nav group visible at a time
- `>= 768px`: persistent primary navigation is allowed
- command-center and other landing pages may use a secondary rail only when the primary content column still remains dominant
- pages must never require simultaneous visibility of sidebar, secondary rail, and modal to complete a primary task

Downstream workspace changes may introduce additional larger breakpoints for multi-pane layouts, but they must inherit this shell contract rather than inventing parallel navigation models.

## Open Questions

- Should the scope selector offer a first-class `All nodes` mode from day one, or only `Hub + individual agents` until more cross-node pages are rebuilt?
- Which Command Center sections should be fully API-backed in phase 1 versus stubbed behind existing data until later changes land?
