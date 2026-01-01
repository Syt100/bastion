# Change: Improve error feedback (API details + UI inline messages)

## Why
Today many failures in the Web UI only show a generic “failed” message without the actual reason, which makes troubleshooting and self-service difficult. In some cases, user input validation failures are incorrectly surfaced as HTTP 500 (`internal_error`), preventing the UI from showing actionable guidance.

## What Changes
- Backend: extend the standard error response schema with an optional `details` field for safe, structured diagnostics (e.g. which field is invalid, retry-after seconds).
- Backend: ensure common user input validation failures return correct 4xx status codes with stable `error` codes and helpful `message` + `details` (instead of 500).
- Web UI: for form workflows (login/setup/settings/job editor/etc.), prefer inline error feedback that shows the reason and (when available) highlights the specific field using backend `details`.
- Web UI: map known backend error codes to localized (`zh-CN`/`en-US`) human-friendly messages; fall back to backend `message` when unknown.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - Backend: `crates/bastion/src/http/error.rs` and request handlers that validate user input
  - Web UI: form pages/components that currently show generic error toasts

## Compatibility / Non-Goals
- No API route changes.
- No secrets or sensitive values are included in `details`.
- Login failures should remain non-enumerable (no difference between “user not found” vs “bad password” to the client).

