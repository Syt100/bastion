# Change: Harden error propagation (prevent regressions, safer debug details)

## Why
New features frequently introduce endpoints that accidentally surface user-actionable failures (e.g. permission denied, missing path, not found) as HTTP 500 `internal_error`. This causes the Web UI to show generic “Internal Server Error” and loses the real reason, making troubleshooting slow and error-prone.

## What Changes
- Backend: add a safe, centralized fallback that classifies common root-cause errors (IO permission denied / not found, DB row not found) into stable, user-facing 4xx/403 error codes instead of `internal_error`.
- Backend: add an opt-in “debug error details” mode (off by default) that includes sanitized diagnostic info in `details.debug` for `internal_error` responses to speed up dev troubleshooting without exposing details by default.
- Web UI: standardize toast-style errors to always use the shared error formatter (localized known `error` codes, fallback to backend `message`) to prevent UI regressions.
- Web UI: capture `X-Request-Id` from API responses and surface it for 5xx/internal errors to enable quick log correlation.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - Backend: `crates/bastion-http/src/http/error.rs` and request handlers that currently rely on implicit `?` conversions
  - Hub CLI/config: `crates/bastion-config`, `crates/bastion/src/config.rs`
  - Web UI: `ui/src/lib/api.ts`, `ui/src/lib/errors.ts`, and components that show toasts

## Compatibility / Non-Goals
- No breaking API route changes.
- Debug details are **off by default** and MUST NOT include secrets; the mode is for dev/troubleshooting only.
- This does not replace endpoint-specific validation (those still should return precise `error` codes + `details.field` when applicable); it reduces regressions by providing a better default fallback.

