# Change: Update incomplete cleanup UI title and status help

## Why
The current cleanup page title (“未完成清理”) is ambiguous and can be misunderstood as “cleanup did not finish”.
Also, the cleanup task statuses are operational concepts that are not self-explanatory to users, which increases support cost and slows down troubleshooting.

## What Changes
- Rename the cleanup page title to a clearer label (zh-CN: “不完整运行清理”).
- Add an in-page “?” help dialog that explains the meaning of each cleanup task status.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/` (view + i18n)

## Compatibility / Non-Goals
- No backend behavior changes.
- No status semantics changes; only explanatory UI is added.

