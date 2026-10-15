## Why

After dependency PRs were combined, Cargo.lock required normalization. Platform and contract jobs rejected it with `--locked`, while the main CI script silently rewrote it and passed. This allows an inconsistent committed lockfile to escape the main CI gate.

## What Changes

- Normalize Cargo.lock with the project's Rust 1.92.0 toolchain while preserving dependency versions.
- Add `--locked` to dependency-resolving Cargo commands in `scripts/ci.sh` so CI validates the committed graph.

## Capabilities

### Modified Capabilities
- `ci`: reject dependency resolution that would modify Cargo.lock.

## Impact

No runtime behavior, dependency version policy, or release metadata changes. Existing platform and driver-contract gates already use `--locked`.
