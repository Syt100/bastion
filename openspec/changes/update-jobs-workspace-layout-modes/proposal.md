# Change: Add layout modes and table view to the Jobs workspace (ops-first)

## Why
Operators switch between two high-frequency workflows:

- **Manage many jobs**: search, filter, sort, scan status, and perform quick actions.
- **Inspect one job**: focus on a single job's configuration, latest runs, and data lifecycle details.

The current desktop master-detail layout is good for quick navigation, but it cannot adapt:
- When managing many jobs, the details pane consumes space that is better used by the list.
- When inspecting a job, the jobs list consumes space that is better used by the details.

We want the Jobs workspace to support these workflows without changing routes, without adding new pages, and without degrading the existing mobile experience.

## What Changes
- Add **desktop layout modes** to the Jobs workspace:
  - **Split** (list + detail) for browsing and quick context.
  - **List-only** (full-width list) for filtering and management.
  - **Detail-only** (full-width detail) for focused inspection.
- Add a **List/Table view toggle** for the jobs list:
  - **List view** keeps the current row layout (scanability-first).
  - **Table view** is optimized for ops management (columns + sorting), and is available in **List-only** mode.
- Preserve and protect mobile UX:
  - Mobile remains single-column navigation (jobs list, then job workspace with back affordance).
  - Table view and desktop layout controls are not shown on mobile-sized screens.
- Persist user preferences on desktop (layout mode and list view mode) so operators can keep their preferred working style.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue` (layout modes, list/table toggle)
  - `ui/src/views/jobs/JobWorkspaceView.vue` (detail-only affordances / header controls)
  - `ui/src/stores/ui.ts` (persisted UI preference state)
  - i18n strings for view/mode labels and tooltips
- Backend APIs: no changes required.

## Non-Goals
- Adding a dedicated global snapshots/backup data management page.
- Changing routes, job semantics, scheduling behavior, or retention/snapshot behavior.
- Introducing backend bulk job operations (can be layered later if needed).

