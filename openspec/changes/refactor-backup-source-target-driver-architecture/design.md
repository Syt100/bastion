# Design: Driver-based backup architecture for long-term extensibility

## Context
The existing model is organized around static enums for source and target kinds. This keeps types
simple, but pushes extension logic into many distributed `match` sites.

For long-term maintainability, the architecture should treat source/target kinds as drivers with
stable contracts. Core runtime code should orchestrate drivers, not branch on specific kinds.

## Goals
1. Add new source/target implementations without editing core scheduler/control-plane code paths.
2. Remove source-target feature coupling from execution code and move it to capability negotiation.
3. Keep backward compatibility for existing specs and mixed-version deployments.
4. Unify backup/restore/cleanup semantics behind one target lifecycle.
5. Enforce high confidence through contract tests and matrix tests.

## Non-goals
- Out-of-process plugin loading in initial phases.
- Immediate deletion of V1 spec/protocol models.
- Shipping new external provider drivers as part of this change.

## High-level architecture

### New logical components
- `bastion-driver-api`
  - source and target trait contracts
  - typed capability model
  - shared error taxonomy and operation context structs
- `bastion-driver-registry`
  - registration and lookup by `driver_id` + version
  - capability retrieval for planner
- `ExecutionPlanner`
  - computes execution plan from source/target capabilities + pipeline requirements
- `SpecTranslator`
  - translates `JobSpecV1` into canonical internal `JobSpecV2`

### Data-flow changes
1. API accepts V1 or V2 job spec payload.
2. Validation resolves to canonical internal V2 representation.
3. Scheduler queries registry for source/target drivers.
4. Planner emits a deterministic execution plan.
5. Runtime executes plan via driver contracts.
6. Restore/artifact stream/cleanup use the same target driver lifecycle.

## Contract design

### Source driver contract (conceptual)
- `validate(config, env) -> ValidationResult`
- `capabilities() -> SourceCapabilities`
- `prepare_run(ctx, config) -> PreparedSource`
- `build_artifacts(ctx, prepared_source, pipeline, writer) -> BuildSummary`

### Target driver contract (conceptual)
- `validate(config, auth_refs, env) -> ValidationResult`
- `capabilities() -> TargetCapabilities`
- `open_writer(ctx, run_identity, pipeline) -> TargetWriter`
- `open_reader(ctx, run_identity) -> TargetReader`
- `cleanup_run(ctx, run_identity) -> CleanupResult`
- `snapshot_redacted(ctx, config, auth_refs) -> serde_json::Value`

### Capability model
Capabilities are explicit and versioned. Examples:
- `supports_archive_rolling_upload`
- `supports_raw_tree_direct_upload`
- `supports_resume_by_size`
- `supports_random_read`
- `supports_server_side_cleanup`
- `requires_local_staging`

## Job spec model

### Canonical internal model (`JobSpecV2`)
- `source`
  - `type`
  - `version`
  - `config` (opaque json validated by source driver)
- `target`
  - `type`
  - `version`
  - `config` (opaque json validated by target driver)
  - `auth_refs` (secret references only)
- `pipeline`
  - existing format/encryption + future extensible options

### Compatibility
- V1 payloads are parsed and translated to V2 before execution.
- Translation preserves behavior parity and legacy defaults.
- Existing storage rows and APIs remain accepted during migration.

## Planner design

### Inputs
- canonical job spec
- source and target capabilities
- pipeline requirements
- policy flags (consistency, upload-on-failure, etc.)

### Output
A plan with explicit stages, e.g.:
- source read strategy
- packaging strategy
- upload strategy
- finalize strategy
- failure cleanup strategy

### Determinism
Planner output must be deterministic for the same inputs. Plan details are logged in run events and
attached to run summary for debuggability.

## Unified target lifecycle use cases

### Backup
`open_writer -> upload artifacts/parts -> finalize` (or `abort` on failure)

### Restore and artifact stream
`open_reader` supplies manifest/index/payload/raw-tree reads for both restore and streaming APIs.

### Incomplete cleanup
`cleanup_run` owns target-specific deletion and retry classification.

### Run snapshot persistence
`snapshot_redacted` provides normalized redacted target snapshot values used by run artifacts.

## Security model
- Job specs MUST not contain inline credentials for targets.
- All credentials must be secret references resolved in a node-scoped context.
- Driver validation gets secrets through controlled resolver APIs only.
- Redacted snapshot serialization is mandatory and centrally audited.

## Observability
- Standard labels/tags in events and metrics:
  - `source_driver`, `source_driver_version`
  - `target_driver`, `target_driver_version`
  - `plan_mode`, `plan_fallback_reason`
- Planner decision and fallback reasons are emitted as structured event fields.

## Testing strategy

### Contract tests
Each driver must pass shared contract suites:
- validation contract
- writer/read/finalize/abort contract
- cleanup idempotency contract
- error classification contract

### Matrix tests
Cross-product matrix by:
- source driver x target driver
- artifact format
- encryption mode
- critical planner policy combinations

### Compatibility tests
- V1 and V2 specs produce equivalent behavior for legacy configurations.
- Mixed Hub-Agent versions negotiate protocol fields safely.

## Migration plan

### Phase 1: Introduce architecture skeleton
- Add driver API and registry with adapters around existing implementations.
- Add planner in shadow mode (decision logged, not authoritative).

### Phase 2: Switch execution to planner + driver calls
- Move scheduler and agent execution to planner output.
- Remove direct source-target conditional branches from core execution paths.

### Phase 3: Unify target lifecycle consumers
- Migrate restore, artifact stream, run snapshot, and incomplete cleanup to target lifecycle.

### Phase 4: Promote V2 and deprecate V1 internals
- Keep API compatibility but reduce internal V1 branching.
- Document deprecation timeline.

## Risks and mitigations
- Risk: migration regressions due to broad surface area.
  - Mitigation: adapters + phased rollout + parity tests.
- Risk: capability model too narrow for future drivers.
  - Mitigation: versioned capability schema and extension fields.
- Risk: planner introduces opaque behavior.
  - Mitigation: deterministic outputs + structured decision logs.
