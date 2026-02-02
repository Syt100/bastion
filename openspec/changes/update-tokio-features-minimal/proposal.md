# Change: Minimize tokio feature flags across the Rust workspace

## Why
Several crates currently depend on `tokio` with `features = ["full"]`, which increases compile times and can inflate release binary size. The codebase only uses a subset of Tokio, so we can explicitly enable just the required feature flags.

## What Changes
- Replace `tokio` `features = ["full"]` with a minimal set of feature flags in workspace member crates.
- Add a lightweight CI guard that fails when `tokio/full` is reintroduced in any crate manifest.
- Keep runtime behavior unchanged (this is a build/dependency configuration optimization).

## Impact
- Affected specs: `dev-workflow`
- Affected code: `crates/*/Cargo.toml`, `scripts/ci.sh` (and `scripts/ci.ps1` if applicable)

