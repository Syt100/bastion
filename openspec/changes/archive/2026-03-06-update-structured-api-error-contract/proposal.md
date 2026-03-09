# Change: Update structured API error contract and UI mapping

## Why
Recent regressions show the same root issue across multiple pages:

- The backend reuses one error code for multiple meanings (for example `invalid_password` can mean "required" or "too short").
- The Web UI often maps messages using only `error` or even by parsing English `message` text.
- As endpoints evolve, this leads to incorrect field feedback, fragile logic, and repeated one-off hotfixes.

The project needs a durable error contract where machine-readable semantics are first-class, and UI rendering is fully driven by structured data rather than message strings.

## What Changes
- Define and enforce a structured API error details model across backend handlers:
  - keep top-level `error` and `message`
  - standardize `details.reason`, `details.field`, `details.params`
  - support `details.violations[]` for multi-field validation cases
- Migrate high-risk validation/error paths to explicit `reason` values (setup auth, WeCom/SMTP secrets, runtime config, FS/WebDAV browse flows).
- Extend agent FS list error transport to include structured error code/details (similar to current WebDAV flow) so the hub/UI do not infer behavior from message substrings.
- Refactor Web UI error handling to one shared resolver:
  - translate using `apiErrors.<code>.<reason>` first
  - fallback to `apiErrors.<code>`
  - fallback to backend `message`
- Refactor form field error mapping to shared helpers driven by `field + reason + params`, replacing ad-hoc per-page `if` chains.
- Add contract-level regression tests (backend + frontend) for `code/reason/params/field` semantics.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected backend areas:
  - `crates/bastion-http/src/http/error.rs` (error details helpers/shape)
  - request handlers currently emitting ambiguous codes/messages
  - agent transport plumbing for FS list responses (`bastion-core`, `bastion`, `bastion-http`, `bastion-engine`)
- Affected frontend areas:
  - `ui/src/lib/errors.ts` and tests
  - form pages currently doing code-only/manual mapping
  - path picker data sources currently doing message substring classification
  - locale dictionaries for new `apiErrors.<code>.<reason>` keys

## Compatibility / Non-Goals
- No route-path changes.
- Keep existing top-level response shape (`error`, `message`, `details`) for compatibility.
- Existing clients that only read `error/message` remain functional.
- This change does not require full migration of every historical endpoint in one pass; it requires high-risk/user-facing paths to be migrated and establishes the new required pattern for all new endpoints.
