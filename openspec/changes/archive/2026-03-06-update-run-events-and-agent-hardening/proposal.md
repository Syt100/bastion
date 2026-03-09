# Change: Run events UI performance and agent/hub hardening

## Why
- The Run Events viewer currently does an initial fetch and then reconnects without `after_seq`, causing redundant event delivery and wasted work.
- Long-running or verbose runs can produce many events; rendering large lists with inline JSON can become slow and hard to navigate.
- Agent offline run persistence and sync currently relies on blocking filesystem I/O in async hot paths.
- The Agent run ingest endpoint should be more defensive (size limits, stricter validation) and support idempotent upsert semantics.

## What Changes
- Web UI: use `after_seq` for run events WebSocket catch-up, add follow mode, and optimize event rendering for large runs (fields/details shown on demand).
- Backend: enforce request body size limits, strengthen `/agent/runs/ingest` validation, and upsert run metadata on conflict.
- Hub↔Agent: make config snapshot IDs deterministic for identical content and avoid re-sending unchanged snapshots to connected Agents.
- Agent: move offline run persistence/sync file I/O off the async hot path.
- Refactor: split the Agent client implementation into focused modules to keep complexity manageable.

## Impact
- Affected areas: `bastion-http`, `bastion-engine`, `bastion`, `bastion-core`, `ui`
- Protocol: no protocol version bump (stay on v1); changes are additive/behavioral.
- UX: improved responsiveness and usability of the run events viewer; no functional regressions expected.

