# Design: Remediate CodeQL secret-hygiene alerts

## Scope
- Test-only credential literals used in auth/keypack setup.
- Test assertions and panic messages that may emit secret values.

## Goals
- Eliminate current open alerts for `rust/hard-coded-cryptographic-value` and `rust/cleartext-logging`.
- Keep tests deterministic enough for CI and avoid semantic changes in production logic.
- Make secure test patterns easy to reuse.

## Non-goals
- Changing production auth/password policy logic.
- Relaxing CodeQL or disabling security checks globally.

## Design Decisions

### 1) Runtime-generated test passphrases
- Replace literal password strings in tests with locally generated passphrases (`Uuid::new_v4().to_string()` etc.).
- Keep passphrase generation inside test scope so each test remains isolated.

### 2) Secret-safe assertions
- Replace `assert_eq!`/debug panics on secret-bearing buffers/structs with boolean assertions and fixed error messages.
- Avoid formatting sensitive variants with `{:?}` in panic paths.

### 3) Regression guard intent
- Normalize a small helper style in affected test modules to avoid reintroducing hard-coded secret literals.

## Testing Strategy
- Run targeted crate tests for touched test modules.
- Run `bash scripts/ci.sh` to ensure all quality gates pass.
- Validate change spec via `openspec validate --strict`.
