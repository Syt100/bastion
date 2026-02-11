# Change: Final decoupling for restore target readers

## Why
The previous driver-platform refactor removed most source/target coupling, but restore-related paths
still contain target-kind branches and duplicated target config mapping logic.

Current gaps:
- `bastion-driver-registry::open_reader` still switches on target kind internally.
- Restore access and Hub artifact stream still branch directly on concrete reader variants.
- Hub/Agent/runtime target config building (`DriverId + config`) is duplicated across modules.

This keeps extension cost higher than necessary when adding a new target.

## What Changes

### 1) Move reader creation into target drivers
- Extend `TargetDriver` with reader creation contract (`open_reader`).
- Introduce a reader trait contract (`TargetRunReader`) for run-relative artifact operations.
- Make registry delegate `open_reader` to drivers instead of matching on target kind.

### 2) Make restore + artifact stream consume reader contracts
- Refactor restore access and entries index fetch to use reader trait methods.
- Refactor restore/verify artifact source creation to use a driver-backed source adapter.
- Refactor Hub artifact stream to use generic reader operations and keep only local-agent fast path branching.

### 3) Centralize target runtime config resolution
- Add shared helpers in `bastion-driver-registry` for:
  - `job_spec::TargetV1` -> `(DriverId, runtime config)`
  - `TargetResolvedV1` -> `(DriverId, runtime config)`
  - snapshot input config derivation
- Migrate Hub store, Agent store, planner target-id mapping, snapshot builder, restore reader open,
  and artifact stream reader open to these helpers.

### 4) Add contract tests and docs
- Add/extend contract tests for reader behavior (`complete`, `read_bytes`, `head_size`, `get_to_file`, local path hint).
- Add tests for shared target runtime config helper validation.
- Sync `docs/dev/driver-platform.md` and `docs/zh/dev/driver-platform.md` with the final reader contract workflow.

## Impact
Affected capabilities:
- `restore`
- `targets`

Affected components:
- `crates/bastion-driver-api`
- `crates/bastion-driver-registry`
- `crates/bastion-backup`
- `crates/bastion-http`
- `crates/bastion-engine`
- `crates/bastion`

## Compatibility
- Backward-compatible for existing local_dir/webdav jobs.
- No API contract changes for end users; internal extension path is simplified.
