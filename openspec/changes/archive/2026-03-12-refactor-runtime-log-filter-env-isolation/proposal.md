# Change: Refactor runtime log filter environment isolation

## Why
PR `#7` exposed a pre-existing flaky backend test: runtime config resolution tests mutate and observe the process-wide `RUST_LOG` environment variable from the same test binary. Because Rust tests run in parallel by default, one test can temporarily set `RUST_LOG` while another test expects the DB fallback path, causing nondeterministic failures.

This is a structural problem, not a UI dependency problem. The current implementation also reads `RUST_LOG` from inside helper functions, which makes the resolution path harder to test as a pure unit.

## What Changes
- Capture `RUST_LOG` once at runtime entry points and thread it through runtime-config resolution as explicit input.
- Make runtime log-filter resolution and logging filter construction pure with respect to process environment.
- Update backend tests to stop mutating process-global `RUST_LOG`.
- Add regression coverage for explicit env-input precedence and DB fallback behavior without relying on shared process state.

## Impact
- Affected specs: `backend`
- Affected code (representative):
  - `crates/bastion/src/main.rs`
  - `crates/bastion/src/logging/mod.rs`
  - `crates/bastion/src/logging/tests.rs`

## Non-Goals
- Changing user-visible log filter precedence.
- Changing CLI flags or environment variable names.
- Serializing the full backend test suite in CI.
