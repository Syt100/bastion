## Why

The backend still has two runtime-configuration paths that rely on hidden process-global state: HTTP internal-error rendering uses a process-global debug flag, and filesystem snapshot preparation reads environment variables inside a lower-level helper. Both patterns make behavior harder to reason about, make tests less isolated, and leave room for cross-request or cross-test interference in the future.

## What Changes

- Refactor HTTP internal-error rendering to use router/request-scoped render options instead of a process-global mutable flag.
- Refactor filesystem snapshot preparation to capture snapshot runtime settings at execution entry points and pass them explicitly into snapshot resolution helpers.
- Replace regression coverage that depends on hidden global state with explicit-input tests for both HTTP error rendering and snapshot settings.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `backend`: backend runtime configuration and error rendering requirements now require explicit runtime inputs instead of hidden process-global state.

## Impact

- Affected code:
  - `crates/bastion-http/src/http/error.rs`
  - `crates/bastion-http/src/http/middleware.rs`
  - `crates/bastion-http/src/http/mod.rs`
  - `crates/bastion-backup/src/backup/filesystem/source_snapshot.rs`
  - `crates/bastion-engine/src/scheduler/worker/execute/filesystem.rs`
  - `crates/bastion/src/agent_client/tasks/filesystem.rs`
- No API or CLI contract changes.
- No new dependencies.
