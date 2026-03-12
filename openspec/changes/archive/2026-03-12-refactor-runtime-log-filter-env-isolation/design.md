## Context
`resolve_log_filter` currently reads `RUST_LOG` directly, and `logging::build_filter` does the same as a second fallback. Unit tests in `crates/bastion/src/main.rs` therefore depend on ambient process environment. One test already uses a mutex guard before setting `RUST_LOG`, but the neighboring test that expects DB precedence does not participate in the same guard and does not explicitly clear the variable. Under parallel execution this produces a flaky failure.

## Goals / Non-Goals
- Goals:
  - Remove direct `RUST_LOG` reads from pure resolution helpers.
  - Ensure runtime entry points remain the only place that touches process environment for log-filter fallback.
  - Make unit tests deterministic under parallel execution.
- Non-Goals:
  - Redesign the logging subsystem.
  - Change log filter precedence or defaults.

## Decisions
- Decision: introduce a small explicit runtime environment input for runtime-config resolution.
  - Rationale: callers can capture `RUST_LOG` once and pass it down, which keeps helper behavior deterministic and test-friendly.
- Decision: when `RUST_LOG` wins precedence, write the resolved filter into `effective_logging_args.log` before calling logging initialization.
  - Rationale: this preserves existing runtime behavior while allowing `logging::build_filter` to stop reading process environment directly.
- Decision: replace environment-mutation tests with explicit-input tests.
  - Rationale: unit tests should validate precedence rules without sharing mutable global state.
- Decision: keep the change localized to `crates/bastion`.
  - Rationale: the bug is in backend runtime config/logging resolution; no broader API or UI changes are needed.

## Risks / Trade-offs
- Introducing a new input struct or parameter increases call-site churn.
  - Mitigation: keep the structure minimal and only include `rust_log`.
- Removing fallback reads from `logging::build_filter` could break callers that bypass runtime-config resolution.
  - Mitigation: inspect current call sites and ensure the runtime entry points always materialize the resolved filter into `LoggingArgs`.

## Migration Plan
1. Add explicit runtime environment capture/input at backend entry points.
2. Refactor `resolve_log_filter` and `resolve_hub_runtime_config_meta` to consume explicit env input.
3. Refactor logging filter construction to rely on resolved `LoggingArgs` instead of raw process env.
4. Replace env-mutation tests with explicit-input tests and add regression coverage.
5. Validate with targeted backend tests and OpenSpec strict validation.

## Open Questions
- None.
