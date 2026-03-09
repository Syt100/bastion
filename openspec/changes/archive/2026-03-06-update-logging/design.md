## Context
Bastion uses `tracing` for structured logging. The current subscriber is initialized with `EnvFilter::from_default_env()`, which can produce no visible logs when no filter is configured.

## Goals / Non-Goals
- Goals:
  - Make `INFO` logs visible by default
  - Keep logs actionable without being overly chatty
  - Support optional file logging with rotation for production deployments
  - Avoid leaking sensitive data into logs
- Non-Goals:
  - Distributed tracing / OTLP exporters
  - Log ingestion/analytics integrations

## Decisions
- Decision: Continue using `tracing` / `tracing-subscriber` and add `tracing-appender` for rolling file output.
- Decision: Keep console logging as default; file logging is opt-in via env/CLI.
- Decision: Use a conservative default filter that suppresses very noisy dependency logs (e.g. HTTP access logs) unless explicitly enabled.

## Risks / Trade-offs
- File rotation policies vary by environment; to stay portable, rotation uses time-based rolling (hourly/daily/never).
- Retention cleanup may delete logs if misconfigured; defaults should be safe and the feature must be configurable/disable-able.

## Open Questions
- None (initial implementation will ship with time-based rotation and configurable retention-by-count).
