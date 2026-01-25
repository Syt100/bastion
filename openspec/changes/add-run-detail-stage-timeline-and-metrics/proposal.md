# Change: Run Detail Stage Timeline + Rich Progress Metrics

## Why
Run progress is currently hard to interpret after-the-fact: users cannot easily answer "how long did each stage take", "where did it fail", or "what speed did it reach".

## What Changes
- Add a stage timeline showing Scan / Build(Packaging) / Upload stage durations and total duration.
- Improve progress metrics readability:
  - Distinguish source size vs transfer size.
  - Keep a meaningful final transfer speed visible after completion.
  - Optionally show peak speed during the run (when available).
- Improve failure diagnostics:
  - Indicate which stage the run failed in.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/src/components/runs/RunProgressPanel.vue` and run detail helpers
- No backend changes
