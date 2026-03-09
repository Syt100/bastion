# Design: Structured API error contract

## Context
Current error handling is partially structured (`error`, `message`, optional `details`) but semantically inconsistent:

- Some endpoints encode machine meaning in message text.
- Same code can represent different validation reasons.
- Frontend still uses message-substring heuristics in some flows.

This design standardizes machine-readable semantics while retaining backward compatibility.

## Goals
- Make error semantics explicit and stable for UI logic.
- Remove dependency on localized/English message text for behavior decisions.
- Keep wire compatibility for existing clients reading `error/message` only.
- Provide a migration path that can be rolled out incrementally per endpoint.

## Non-Goals
- Full replacement of every historical error code in one release.
- Removal of human-readable `message` from responses.

## Contract shape
Top-level response remains:

```json
{
  "error": "invalid_password",
  "message": "Password must be at least 12 characters",
  "details": {
    "reason": "min_length",
    "field": "password",
    "params": {
      "min_length": 12
    }
  }
}
```

### `details` schema
- `reason?: string`  
  Machine-readable sub-code under `error`.
- `field?: string`  
  Primary field name for single-field validation errors.
- `params?: object`  
  Parameter bag for templated messages (e.g. `min_length`, `max`, `index`).
- `violations?: Array<{ field: string, reason: string, params?: object, message?: string }>`  
  Optional multi-field validation list.

## Backend design

### Error builder helpers
Introduce helper methods on `AppError` (or companion helpers) to reduce manual JSON assembly:

- `with_reason(reason: &'static str)`
- `with_field(field: &'static str)`
- `with_param(key, value)`
- `with_violation(field, reason, params)`

These helpers merge into `details` instead of replacing it.

### Code/reason policy
- `error` remains stable top-level category.
- `reason` expresses specific cause branch.
- New behavior: when a code has multiple meanings, handlers MUST set `reason`.
- Existing message remains human-readable fallback only.

### Agent FS/WebDAV transport
Align FS list response with WebDAV style:

- Add `error_code?: string` for FS list protocol message.
- Optionally add `error_details?: serde_json::Value` for reason/field/params propagation.
- Hub mapping uses structured error fields first; message fallback only for unknown legacy agents.

## Frontend design

### Shared resolver
Enhance `toApiErrorInfo`:

1. Read `code`, `reason`, `field`, `params`, `violations` from error body details.
2. Localization lookup order:
   - `apiErrors.${code}.${reason}` with params
   - `apiErrors.${code}` with params when applicable
   - backend `message`
3. Preserve request-id suffix behavior for 5xx.

### Shared field mapper
Add utility:

- input: `ApiErrorInfo`, field-to-i18n mapping table or resolver callback
- behavior:
  - if `violations` exists, apply each violation by `field+reason`
  - else use single `field+reason`
  - fallback to generic `code` mapping

This removes duplicated per-view mapping logic and keeps behavior consistent.

### Path picker mapping
Replace message substring matching in FS picker with structured mapping from `code` (and optionally `reason`) to picker error kinds.

## Migration strategy

### Phase 1: infrastructure
- add backend helpers + frontend resolver support for `reason/params`
- keep old endpoints functional (no forced `reason` yet)

### Phase 2: high-risk endpoint migration
- setup/auth password validation
- WeCom/SMTP destination validation
- hub runtime config validation
- FS/WebDAV list failure mapping

### Phase 3: guardrails
- tests and lint-like checks to enforce structured reason for multi-meaning codes
- PR guidance: new validation errors require `reason`

## Testing strategy

### Backend
- unit tests for helper merging behavior
- handler tests asserting `error + details.reason + details.field + details.params`
- protocol tests for FS list `error_code` and mapping coverage

### Frontend
- unit tests for resolver fallback chain and parameterized translations
- unit tests for field mapper with `violations[]`
- picker mapping tests proving no message-substring dependency

## Risks and mitigations
- Risk: partial rollout leaves mixed styles.  
  Mitigation: fallback chain keeps old behavior while migrated endpoints gain precision.
- Risk: translation keys drift.  
  Mitigation: test helper that detects unresolved `apiErrors.<code>.<reason>` keys in covered flows.
- Risk: agent protocol compatibility.  
  Mitigation: keep message fallback for older agents; use new fields when present.
