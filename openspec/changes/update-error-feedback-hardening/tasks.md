## 1. Spec
- [x] 1.1 Add `backend` and `web-ui` spec deltas for hardened error propagation
- [x] 1.2 Run `openspec validate update-error-feedback-hardening --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - error classification + debug details flag
- [x] 2.1 Add `debug_errors` hub config flag (CLI + env `BASTION_DEBUG_ERRORS`, default off)
- [x] 2.2 Wire `debug_errors` into `bastion-http` error handling (global toggle)
- [x] 2.3 Classify common root-cause errors in `AppError` fallback:
  - `std::io::ErrorKind::PermissionDenied` -> 403 `permission_denied`
  - `std::io::ErrorKind::NotFound` -> 404 `path_not_found`
  - `sqlx::Error::RowNotFound` -> 404 `not_found`
- [x] 2.4 When debug mode is enabled, include `details.debug` for `internal_error` only
- [x] 2.5 Add/adjust tests for error classification and debug gating
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test`
- [x] 2.7 Commit backend changes (detailed message)

## 3. Web UI - request id + standardized toast formatting
- [x] 3.1 Capture `X-Request-Id` in `apiFetch()` and attach it to thrown `ApiError`
- [x] 3.2 Extend shared error formatting to optionally surface Request ID for internal/5xx errors
- [x] 3.3 Replace ad-hoc toast error handling (`error.message`) with shared formatter in components that still do this
- [x] 3.4 Add i18n strings for Request ID label (and any new error codes)
- [x] 3.5 Run `npm --prefix ui run lint`, `npm --prefix ui run test`, `npm --prefix ui run build`
- [x] 3.6 Commit Web UI changes (detailed message)
