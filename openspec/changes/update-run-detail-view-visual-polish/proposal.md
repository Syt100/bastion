# Change: Improve Run Detail visual hierarchy and readability

## Why
The Run Detail page currently shows the right data, but the information hierarchy is weak: primary signals (status, duration, target, warnings/errors, progress) are visually diluted, empty sections consume too much space, and events/summary are hard to scan.

## What Changes
- Improve header hierarchy:
  - show status badge near the title
  - treat `run_id` as secondary information with a one-click copy affordance
  - keep a single primary action (Restore) and move secondary actions into an overflow menu on desktop
- Rework the “overview + progress” first screen layout:
  - on desktop: two-column layout with balanced card height and consistent spacing
  - on mobile: single-column layout with reduced visual noise
- Make Operations and Events sections more readable:
  - compact empty-state for Operations (avoid a large empty table)
  - replace the events DataTable with a timeline-style list (single-line message, click/hover for details)
- Present Summary as “structured highlights + raw JSON (collapsed)”, with copy affordance.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/RunDetailView.vue`
  - Shared UI components for header/layout and events list (as needed)

## Non-Goals
- Changing backend APIs, run/operation schemas, or progress semantics.
- Adding new run actions or modifying restore/verify workflows.

