## 1. Spec and contract definition
- [x] 1.1 Add `backend` and `web-ui` spec deltas for structured error contract and mapping rules
- [x] 1.2 Add `design.md` documenting schema, compatibility strategy, and migration sequencing
- [x] 1.3 Run `openspec validate update-structured-api-error-contract --strict`

## 2. Backend: structured error payloads and transport
- [x] 2.1 Introduce reusable helpers/builders for `details.reason`, `details.field`, `details.params`, and optional `details.violations`
- [x] 2.2 Migrate setup/auth and secret validation handlers to emit structured reasoned errors (replace ambiguous message-only variants)
- [x] 2.3 Migrate hub runtime config validation errors to structured reason + params
- [x] 2.4 Extend agent FS list protocol response to include `error_code` (and optional error details payload) alongside message
- [x] 2.5 Update hub-side FS/WebDAV error mapping to use structured codes/details instead of message parsing
- [x] 2.6 Add backend tests asserting code/reason/field/params semantics and FS/WebDAV mapping behavior

## 3. Web UI: shared resolver and field mapping
- [x] 3.1 Refactor `toApiErrorInfo` into code+reason aware resolver with ordered fallback (`code.reason` -> `code` -> backend message)
- [x] 3.2 Add shared field-error extraction helper that supports both single-field and `violations[]`
- [x] 3.3 Refactor Setup/Notifications/RuntimeConfig forms to use shared field-error mapping helper
- [x] 3.4 Refactor path picker FS/WebDAV error-kind mapping to rely on structured code/reason only (remove message substring heuristics)
- [x] 3.5 Add locale entries for new `apiErrors.<code>.<reason>` keys in both `en-US` and `zh-CN`
- [x] 3.6 Add frontend unit tests for resolver fallback chain and field mapping semantics

## 4. Verification and quality gates
- [x] 4.1 Run backend checks (`cargo fmt`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test`)
- [x] 4.2 Run UI checks (`npm --prefix ui run lint`, `npm --prefix ui run test`, `npm --prefix ui run build`)
- [x] 4.3 Run project CI script `scripts/ci.sh`
