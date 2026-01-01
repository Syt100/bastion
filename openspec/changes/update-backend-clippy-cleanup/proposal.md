# Change: Update backend clippy cleanliness (no warnings)

## Why
`cargo clippy` currently emits a number of warnings, which makes CI output noisy and can hide real regressions.

## What Changes
- Fix clippy warnings in the backend by adopting idiomatic patterns (derived defaults, `clamp`, `is_multiple_of`, let-chains, etc.)
- Reduce overly-large enum variants by introducing indirection where appropriate (no JSON schema changes)
- Update CI scripts to treat clippy warnings as errors (`-D warnings`)

## Impact
- Affected specs: `backend`
- Affected code: backend modules touched by clippy warnings, `scripts/ci.sh`, `scripts/ci.ps1`

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No behavior changes intended; changes are refactors/cleanups to satisfy linting and improve maintainability.

