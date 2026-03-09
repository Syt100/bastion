# Change: Refactor Jobs into a workspace (master-detail) with job-scoped Run Detail drawer

## Why
The current Jobs experience fans out into many related-but-separate pages (Jobs list → Job detail → Runs → Run detail → Snapshots → Retention, etc.).
This causes two UX problems:

- **Context loss:** users frequently lose the “I am working on this job” context when navigating to Run Detail or other subpages.
- **Navigation complexity:** the back/forward mental model becomes unclear, especially on mobile where each hop becomes a full-page transition.

We want to treat Jobs as a single operator “workspace”: pick a job, work within a small set of sections, and open deep details (like a run) as an overlay instead of a separate top-level page.

## What Changes
- Replace the current node-scoped Jobs view with a **Jobs workspace** at `/n/:nodeId/jobs`.
  - **Desktop:** split master-detail layout (jobs list on the left, selected job workspace on the right).
  - **Mobile:** single-column navigation (jobs list → job workspace), but using the same route structure.
- Replace the current Job detail tab set with **exactly three top-level sections**:
  - **Overview** (status + key info + primary actions)
  - **History** (runs list)
  - **Data** (snapshots + retention policy in one place)
- Refactor Run Detail to be a **job-scoped overlay route** rendered as a drawer:
  - **Desktop:** side drawer overlay.
  - **Mobile:** full-screen drawer overlay.
- Move “advanced/inspect” functionality (e.g. JSON view) behind a “More” menu instead of a top-level job tab.
- Remove the top-level Run Detail page route (`/n/:nodeId/runs/:runId`) and update all in-app links to the new job-scoped Run Detail drawer routes.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/router/index.ts`
  - `ui/src/views/JobsView.vue` (replaced by workspace shell)
  - `ui/src/views/jobs/*` (new sections + overlay)
  - `ui/src/views/RunDetailView.vue` (refactored into embeddable panel + drawer overlay)
  - Pages that link to runs (e.g. Dashboard, Snapshots)
- Backend APIs: no changes (reuse existing `/api/jobs/*` and `/api/runs/*` endpoints).

## Non-Goals
- Preserving old URLs or implementing redirects for legacy routes (explicitly not required).
- Changing backend schemas, scheduling semantics, or run/snapshot/retention behavior.
- Redesigning non-Jobs areas (Settings, Agents) beyond updating links that point to Run Detail.

