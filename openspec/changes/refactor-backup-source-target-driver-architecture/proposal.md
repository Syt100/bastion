# Change: Refactor backup source/target architecture into a driver platform

## Why
The current backend can be extended, but extension cost is high because source and target types are
hard-coded across many modules.

Today, adding a new source or target requires touching many `match` branches across:
- job spec model and validation
- scheduler execution and agent execution
- target upload + rolling upload
- restore and artifact streaming
- run artifact snapshot parsing and incomplete cleanup

This raises long-term risk:
- frequent cross-module regressions when adding a driver
- source-target feature coupling through special-case conditionals
- duplicated logic between Hub and Agent paths
- slower delivery for new targets/sources

## What Changes

### 1) Introduce a first-class driver architecture
- Add a shared `driver-api` contract for source and target drivers.
- Add a runtime driver registry for discovery and invocation.
- Move source/target behavior behind capability-based trait interfaces.

### 2) Introduce execution planning via capability negotiation
- Add an `ExecutionPlanner` that resolves strategy from source/target/pipeline capabilities.
- Replace ad-hoc source-target special-casing with plan-based execution modes.

### 3) Upgrade job spec model to driver envelopes (with compatibility)
- Introduce `JobSpecV2` envelope fields:
  - `source: { type, version, config }`
  - `target: { type, version, config, auth_refs }`
- Keep full backward compatibility by translating V1 specs into V2 at runtime.

### 4) Unify target lifecycle across backup, restore, and cleanup
Each target driver must expose one lifecycle covering:
- write/upload and finalize/abort
- read access for restore and artifact stream
- incomplete-run cleanup
- redacted snapshot serialization for run artifacts

### 5) Standardize security, observability, and test contracts
- Keep credentials referenced by secret refs; no inline credential payload in job specs.
- Standardize driver observability dimensions (`driver`, `capability`, `plan_mode`).
- Introduce mandatory driver contract test suites and matrix tests.

## Impact
Affected capabilities:
- `backup-jobs`
- `backup-runtime`
- `targets`
- `hub-agent`
- `restore`
- `observability`

Affected components (expected):
- `crates/bastion-core` (job spec and protocol models)
- `crates/bastion-engine` (planner, registry, scheduler workers)
- `crates/bastion-backup` (source runtime + restore integration)
- `crates/bastion-targets` (target drivers)
- `crates/bastion` (agent runtime)
- `crates/bastion-http` and `crates/bastion-storage` (validation, run snapshot, cleanup)

## Compatibility and Rollout
- Rollout MUST be phased and backward-compatible.
- Existing jobs (`JobSpecV1`) MUST continue to run without migration downtime.
- Hub/Agent protocol changes MUST support mixed-version clusters during rollout.

## Non-goals
- This change does not add specific new source/target products (such as S3) yet.
- This change does not remove V1 job specs immediately.
- This change does not require out-of-process plugin loading in phase 1.
