# Change: Improve Jobs workspace scanability, density, and safety cues

## Why
The Jobs workspace has been simplified structurally (workspace + 3 sections), but day-to-day operator workflows still have friction:

- Scanability: on the Jobs list, users cannot quickly identify which jobs are failing or when a job last ran.
- Configuration confidence: in Overview, users need to confirm what the job is configured to do (source/target/format/encryption) and how it runs (schedule/timezone/overlap) without opening the editor.
- Vertical density: sections sometimes consume unnecessary vertical space, especially on mobile.
- Safety: high-impact actions in Data (retention apply, bulk delete) should communicate risk and scope clearly.
- Nested scrolling clarity: with pane-scoped scrolling, users need subtle cues that a pane is scrollable and where they are in it.

## What Changes
- Overview
  - Add a compact "run policy" strip (schedule + timezone + overlap) using chips/tags.
  - Improve metadata card presentation: compact cards, larger value typography, consistent tag semantics.
  - Display user-friendly labels for backup format and encryption with optional code details.

- Jobs list
  - Show each job's latest run status and time directly in the list row (compact badge/tag).

- History/Data
  - Add quick status filters (chips) for History to reduce navigation and scrolling.
  - Add guardrail UI for destructive actions in Data (retention apply, bulk delete), including clear warning text and scope hints.

- Workbench scroll cues
  - Add subtle top/bottom scroll shadows (or gradient fades) inside scroll containers (jobs list pane, job section pane) to indicate scrollability.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/jobs/JobOverviewSectionView.vue`
  - `ui/src/views/jobs/JobHistorySectionView.vue`
  - `ui/src/views/jobs/JobDetailRetentionView.vue` and embedded snapshots components
  - Shared UI components/styles (chips, scroll-shadow helper)
  - i18n locales

## Non-Goals
- Changing backend behavior, scheduling semantics, retention behavior, or deletion rules.
- Re-introducing a multi-page Job detail hierarchy; the workspace structure remains.
- Pixel-perfect redesign of unrelated pages.

