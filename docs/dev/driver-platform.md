# Driver Platform Workflow

This page explains how to add new backup **source** or **target** drivers with the current
registry + planner architecture.

## Key modules

- `crates/bastion-driver-api`: shared driver contracts and capability model, including
  `TargetDriver::open_reader` and `TargetRunReader` reader contract.
- `crates/bastion-driver-registry`: driver registration, target lifecycle (`open_writer`,
  `open_reader`, `cleanup_run`, `snapshot_redacted`) and built-in adapters.
- `crates/bastion-driver-registry/src/target_runtime.rs`: shared runtime mapping helpers from
  target specs/resolved targets to `(DriverId, target_config)`.
- `crates/bastion-core/src/execution_planner.rs`: capability-based deterministic planner.
- `crates/bastion-engine/src/scheduler/worker/execute/`: Hub runtime planner integration.
- `crates/bastion/src/agent_client/tasks/`: Agent runtime planner integration.

## Add a new target driver

1. **Define driver id and capabilities**
   - Add a `DriverId` (kind + version) and capability flags in
     `crates/bastion-driver-registry/src/builtins.rs` (or your target registry module).
2. **Implement lifecycle behavior**
   - Implement `TargetDriver::store_run`, `cleanup_run`, and `snapshot_redacted`.
3. **Implement reader contract in the driver**
   - Implement `TargetDriver::open_reader` and return a `TargetRunReader` implementation.
   - Reader methods must cover:
     - `complete_exists`
     - `read_bytes`
     - `head_size`
     - `get_to_file`
     - optional `local_run_dir` hint for node-local fast paths.
4. **Keep snapshots redacted**
   - `snapshot_redacted` output MUST not include raw credentials.
   - Persisted run snapshot shape stays `{ node_id, target }`.
5. **Ensure planner compatibility**
   - Set capability flags so planner can choose direct/rolling/staged modes safely.
6. **Add tests**
   - Add/extend `driver_contract_*` tests in `bastion-driver-registry`.
   - Extend planner matrix tests for supported source-target-format combinations.

## Shared runtime target config mapping

Use shared helpers in `crates/bastion-driver-registry/src/target_runtime.rs` instead of
hand-written per-module mapping:

- `runtime_input_for_job_target(...)`
- `runtime_input_for_resolved_target(...)`
- `snapshot_input_for_job_target(...)`
- `driver_id_for_job_target(...)`

This keeps Hub store, Agent store, planner mapping, snapshot generation, restore reader open, and
artifact stream reader open consistent when new targets are added.

## Restore and artifact stream integration

- Restore and run-entry index APIs read artifacts through `TargetRunReader` contract methods.
- Hub artifact stream uses the same reader contract and only keeps a local-agent fast path branch
  when a reader exposes `local_run_dir` and the run is on a remote agent.
- For non-local readers, stream/restore paths use `head_size + get_to_file` semantics for large
  index/payload artifacts.

## Add a new source driver

1. Add source identity/version mapping (Hub + Agent execution planner adapters).
2. Define source capability flags and planner policy inputs.
3. Implement build pipeline integration in both runtimes (`bastion-engine` and `bastion` agent).
4. Emit planner fields in run events and include `planner` in run summary.
5. Add planner matrix tests for the new source against all supported targets/formats.

## Observability requirements

- Planner event fields must include:
  - `source_driver`
  - `target_driver`
  - `plan_mode`
  - `plan_fallback_reason` (when fallback happens)
- Run summaries should include `planner` payload for success and policy-failure paths.

## CI quality gates

`bash scripts/ci.sh` now runs explicit gates before workspace-wide tests:

- `cargo test -p bastion-driver-registry driver_contract`
- `cargo test -p bastion-core execution_planner_matrix`

CI workflow also has a dedicated `driver-contract-matrix` job with the same checks.
