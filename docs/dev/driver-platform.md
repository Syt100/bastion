# Driver Platform Workflow

This page explains how to add new backup **source** or **target** drivers with the current
registry + planner architecture.

## Key modules

- `crates/bastion-driver-api`: shared driver contracts and capability model.
- `crates/bastion-driver-registry`: driver registration, target lifecycle (`open_writer`,
  `open_reader`, `cleanup_run`, `snapshot_redacted`) and built-in adapters.
- `crates/bastion-core/src/execution_planner.rs`: capability-based deterministic planner.
- `crates/bastion-engine/src/scheduler/worker/execute/`: Hub runtime planner integration.
- `crates/bastion/src/agent_client/tasks/`: Agent runtime planner integration.

## Add a new target driver

1. **Define driver id and capabilities**
   - Add a `DriverId` (kind + version) and capability flags in
     `crates/bastion-driver-registry/src/builtins.rs` (or your target registry module).
2. **Implement lifecycle behavior**
   - Implement `TargetDriver::store_run`, `cleanup_run`, and `snapshot_redacted`.
   - Expose reader wiring via registry `open_reader` so restore and artifact stream use the same
     path.
3. **Keep snapshots redacted**
   - `snapshot_redacted` output MUST not include raw credentials.
   - Persisted run snapshot shape stays `{ node_id, target }`.
4. **Ensure planner compatibility**
   - Set capability flags so planner can choose direct/rolling/staged modes safely.
5. **Add tests**
   - Add/extend `driver_contract_*` tests in `bastion-driver-registry`.
   - Extend planner matrix tests for supported source-target-format combinations.

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
