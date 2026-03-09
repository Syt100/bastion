## Context
The Jobs workspace is an operator-facing workbench. Users repeatedly perform three actions:
1) scan for problems (which jobs failed / last ran),
2) confirm configuration quickly (what will be backed up and how), and
3) take action safely (run now, retention, delete).

This change focuses on information design and interaction density without changing underlying behavior.

## Goals / Non-Goals
- Goals:
  - Improve scanability (status and recency visible in list and overview).
  - Improve configuration confidence in Overview (key settings visible without opening editor).
  - Improve mobile layout density (less vertical waste, no horizontal overflow).
  - Improve safety for destructive actions (warnings and scope cues near actions).
  - Improve clarity of pane-scoped scrolling (scroll shadows/fades).
- Non-Goals:
  - New job types or new target backends.
  - Changes to retention policy logic or deletion guardrails.

## Decisions
- Use a "run policy strip" (chips/tags) instead of additional cards.
  - Rationale: policy data is important but should not compete with configuration cards; chips preserve vertical density.

- Show user-friendly labels with optional code details.
  - Example: show "Archive" plus a small monospace "archive_v1" rather than only the raw enum.

- Apply consistent tag semantics.
  - Security-related signals (encryption) should use success/green when enabled.
  - Status-related signals (run status) should reuse existing run status tag types.

- Add scrollability cues for nested scrollers.
  - Use lightweight shadows/fades that appear only when overflow exists, to avoid visual noise.

## Risks / Trade-offs
- Adding more information to list rows can increase row height.
  - Mitigation: keep badges compact, avoid multi-line by default, and only show the most useful fields.

- Scroll shadows require careful implementation to avoid layout reflow and performance issues.
  - Mitigation: prefer CSS-based approaches when possible; otherwise, throttle scroll listeners and keep DOM minimal.

## Migration Plan
1) Implement UI changes behind existing routes/components (no routing changes).
2) Verify on desktop and mobile:
   - Jobs list scanability.
   - Overview density and readability.
   - History filter chips behavior.
   - Data guardrails and action placement.
   - Scroll shadows appear/disappear appropriately.
3) Run full `scripts/ci.sh`.

