# Change: Improve Dashboard overview (metrics, 7-day trend, recent runs)

## Why
The current Dashboard page is mostly a placeholder and does not help users quickly answer:
- Is the system healthy (Agents online/offline, running/queued work)?
- Are backups succeeding recently?
- What are the most recent runs and where do I click next?

The user request is to keep the Dashboard focused on **overview only** (no checklist), and make it visually polished.

## What Changes
- Backend: add a single aggregated endpoint `GET /api/dashboard/overview` that returns:
  - summary stats (Agents / Jobs / Runs)
  - a 7-day success/failed trend series
  - a recent runs list (click-through to Run Detail)
- Web UI: redesign Dashboard to render the overview with a clean card layout:
  - top KPI cards
  - 7-day trend chart
  - recent runs table/list with status tags and deep links

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - `crates/bastion-http` (new route + handler + tests)
  - `crates/bastion-storage` (if/when query helpers are needed; prefer minimal direct SQL in HTTP layer for this change)
  - `ui/` (Dashboard view + chart component + i18n + tests)

## Non-Goals
- A setup checklist or onboarding flow on the Dashboard.
- New background jobs/queues; this is read-only aggregation.
- Metrics/Prometheus integration (can be a separate change).

