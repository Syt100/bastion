## Context
Rolling upload currently crosses three error domains:
1) synchronous archive writer callback,
2) async uploader task,
3) run terminalization/event persistence.

Today, failures on domain (2) can collapse into channel-drop text in domain (1), then get string-wrapped by archive entry code, which prevents higher layers from classifying root cause.

## Goals / Non-Goals
- Goals:
  - Preserve root cause and context across domain boundaries.
  - Surface structured diagnostics to run events and UI.
  - Provide tunable WebDAV timeout/retry controls for diverse network/proxy environments.
- Non-goals:
  - multipart resumable protocol redesign.

## Decisions
- Decision: introduce a rolling uploader diagnostic bridge shared between callback and uploader task.
  - Rationale: callback can read latest uploader failure context when `blocking_send` fails.
- Decision: await uploader task outcome even if packaging fails.
  - Rationale: avoids losing async failure details and enables deterministic error merge.
- Decision: preserve source chains via `anyhow::Context` instead of string-only `anyhow!(...)` wrappers in archive path.
  - Rationale: enables later classifier to inspect concrete error types in chain.
- Decision: add an explicit failure classifier used at run-finalization point.
  - Rationale: central place to attach `fields` JSON and actionable hint.
- Decision: extend WebDAV request limits with timeout/retry settings as optional knobs.
  - Rationale: long-term operability across gateways with strict body/time limits.

## Risks / Trade-offs
- More detailed errors may expose internal transport details; mitigation: redact URLs and avoid sensitive payloads.
- Additional configuration knobs increase complexity; mitigation: safe defaults and validation bounds.

## Migration Plan
1) Deploy with backward-compatible optional fields.
2) Existing job specs continue to work with defaults.
3) Operators may opt into timeout/retry tuning as needed.

## Open Questions
- Should uploader failure fields also be duplicated in run summary for quick list views (beyond events)?
