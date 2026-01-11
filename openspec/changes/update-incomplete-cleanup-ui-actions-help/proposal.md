# Change: Add incomplete cleanup UI action help

## Why
Operators can see and use action buttons (e.g. “立即重试”, “忽略”) on the incomplete cleanup page, but the effects are not self-explanatory.
This leads to confusion (e.g. why attempts may reset on “立即重试”) and increases support cost.

## What Changes
- Extend the existing “?” help dialog to also explain the effect of each action button:
  - “更多” (details)
  - “立即重试” (re-queue immediately, reset attempts, clear last error)
  - “忽略” (stop auto retry)
  - “取消忽略” (resume auto retry)

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/` (maintenance cleanup view + i18n)

## Compatibility / Non-Goals
- No backend behavior changes.
- No change to status or action semantics; only explanatory UI copy is added.
