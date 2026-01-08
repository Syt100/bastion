# Change: Refactor bastion logging module structure

## Why
`crates/bastion/src/logging.rs` currently bundles logging initialization, log file configuration, rotated-log pruning logic, and suffix parsing/tests in a single file. Splitting it into focused submodules improves readability and makes future maintenance safer.

## What Changes
- Convert `logging` into a folder module under `crates/bastion/src/logging/`
- Split implementation into focused submodules (file config, pruning, suffix parsing, tests)
- Keep the existing public surface stable (`bastion::logging::init`, `LoggingGuard`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/logging.rs`, `crates/bastion/src/logging/*.rs`

## Compatibility / Non-Goals
- No behavior changes intended (log filtering, file rotation configuration, or pruning semantics).

