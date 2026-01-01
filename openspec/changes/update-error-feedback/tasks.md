## 1. Spec
- [x] 1.1 Add `backend` spec delta for error response `details` and 4xx validation semantics
- [x] 1.2 Add `web-ui` spec delta for inline form error feedback using backend error codes/details
- [x] 1.3 Run `openspec validate update-error-feedback --strict`
- [x] 1.4 Commit the spec proposal (detailed message)

## 2. Backend - API error schema + validation fixes
- [x] 2.1 Extend `AppError` JSON body to include optional `details` (safe structured diagnostics)
- [x] 2.2 Ensure invalid input errors return 4xx + stable codes + `details.field` where applicable:
  - WeCom bot webhook URL parse/scheme validation (400 `invalid_webhook_url`)
  - SMTP from/to Mailbox parsing (400 `invalid_from` / `invalid_to`)
  - Agent enrollment token/key parsing (401 `invalid_token` / `unauthorized`)
- [x] 2.3 Add `details.retry_after_seconds` for login rate limiting responses (429 `rate_limited`)
- [x] 2.4 Add/adjust tests for status codes and error body fields (`error`, `message`, `details`)
- [x] 2.5 Run `cargo fmt`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test`
- [x] 2.6 Commit backend error schema + validation changes (detailed message)

## 3. Web UI - Inline errors + i18n mapping
- [x] 3.1 Add a shared helper to normalize `ApiError` into `{ title, message, field? }` using `error`/`message`/`details`
- [x] 3.2 Update form-heavy pages/components to show inline errors (primary) instead of generic “failed” toasts:
  - Login
  - Setup
  - Settings: WebDAV/WeCom bot/SMTP editors
- [x] 3.3 Keep non-form actions (e.g. copy) as toasts, but show meaningful error text when possible
- [x] 3.4 Add i18n keys for known backend error codes in `zh-CN` and `en-US`
- [x] 3.5 Add/adjust UI tests for error mapping and inline rendering where feasible
- [x] 3.6 Run `npm test --prefix ui`
- [x] 3.7 Commit Web UI error feedback changes (detailed message)

## 4. Validation
- [x] 4.1 Run `scripts/ci.sh` (or equivalent) to validate end-to-end checks
