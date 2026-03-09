# Design: Dependabot alert remediation strategy

## Scope
- Open Dependabot alerts currently listed for:
  - Rust: `time`, `bytes`, `rsa`
  - npm: `lodash`, `lodash-es`, `esbuild` (docs toolchain)

## Goals
- Close all currently open runtime alerts where safe and feasible.
- Prefer deterministic lockfile + manifest constraints to avoid regressions.
- Preserve CI stability across Rust/UI/docs pipelines.

## Non-goals
- Broad dependency modernization unrelated to active alerts.
- Refactoring application logic for dependency upgrades.

## Decisions

### 1) Rust resolution hardening
- Keep workspace versions compatible while forcing lockfile updates to patched versions (`time >= 0.3.47`, `bytes >= 1.11.1`).
- Disable `sqlx` default features at workspace level to avoid bringing unused mysql/postgres crypto stack (and related `rsa`) into the graph when only sqlite is needed.

### 2) npm transitive pinning in UI
- Add `overrides` in `ui/package.json` for `lodash` and `lodash-es` to patched versions (`4.17.23`).
- Regenerate `ui/package-lock.json` and validate lint/type-check/tests/build.

### 3) docs esbuild handling
- Try `docs/package.json` override to patched `esbuild` (>= `0.25.0`) and validate docs build.
- Keep the override only if local and CI checks pass; otherwise revert and document upstream constraint risk with next-step recommendation.

## Validation Strategy
- Rust: `cargo fmt --all`, targeted crate tests, full `scripts/ci.sh`.
- UI/docs: ensure install + build/test/lint flows continue to pass through `scripts/ci.sh`.
- Security: verify Dependabot open alert count after push.
