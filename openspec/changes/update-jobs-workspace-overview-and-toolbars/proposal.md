# Change: Move run summary to Overview and compact section toolbars in Jobs workspace

## Why
The Jobs workspace is now structurally simpler (Overview / History / Data + Run drawer), but two UX issues remain:

- **Duplicated/hidden signal:** the run summary cards (history/success/failed/latest run) live in History, even though users often need that signal upfront to decide what to do next.
- **Vertical space waste (especially on mobile):** History/Data currently reserve an extra full row for actions (Refresh, Save), pushing primary content below the fold.

We want the Overview section to act as the job “status dashboard”, and we want section-level actions to live in compact headers rather than consuming a separate row.

## What Changes
- Add an **Overview Run Summary** block (default: **last 7 days**) that includes:
  - Latest run (status + timestamp) with a direct action to open Run Detail (drawer).
  - A compact 7-day summary (total/success/failed, optionally rejected).
- Simplify **History** to be runs-list-first:
  - Remove the large run summary card grid from History.
  - Move History actions (e.g. Refresh) into the list panel header (no standalone action row).
- Compact **Data** section layout:
  - Retention and Snapshots remain in the Data section, but each sub-panel places actions (Save/Refresh/More) in its own header.
  - Avoid standalone action rows that cost vertical space.
- Mobile-first constraints:
  - Overview metrics render as compact chips / small cards without forcing wrap-heavy toolbars.
  - Section header actions collapse to icon/overflow patterns to avoid multi-line headers.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/views/jobs/JobOverviewSectionView.vue`
  - `ui/src/views/jobs/JobHistorySectionView.vue`
  - `ui/src/views/jobs/JobDataSectionView.vue`
  - `ui/src/views/jobs/JobWorkspaceView.vue` (shared data/props if needed)
  - `ui/src/views/JobSnapshotsView.vue`
  - `ui/src/views/jobs/JobDetailRetentionView.vue`
  - i18n strings for new labels (7-day summary)
- Backend APIs: no changes required (derive from existing runs list); optionally can be optimized later.

## Non-Goals
- Changing run/snapshot semantics, retention behavior, or backend schemas.
- Adding a full analytics experience (time range selector, charts) beyond the default 7-day summary.
- Changing Run Detail drawer functionality or actions.
