# Change: Optimize AgentManager pending-request cleanup on disconnect

## Why
`AgentManager::unregister` currently cleans up pending request maps by collecting keys into temporary vectors, and some pending requests are dropped without returning a structured error. This adds avoidable allocations and can surface ambiguous errors (e.g., "channel closed") to callers.

## What Changes
- Replace key-collection loops with in-place draining/removal to reduce allocations.
- Ensure all pending request types complete with a deterministic, explicit error when an agent disconnects.
- Add regression tests covering pending cleanup behavior.

## Impact
- Affected specs: `hub-agent`
- Affected code: `crates/bastion-engine/src/agent_manager.rs`, related callers/tests

