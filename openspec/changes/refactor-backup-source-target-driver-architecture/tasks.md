## 1. Spec
- [x] 1.1 Create proposal, design, and deltas for driver architecture, planning, compatibility, and lifecycle contracts
- [x] 1.2 Run `openspec validate refactor-backup-source-target-driver-architecture --strict`

## 2. Foundation: driver contracts + registry
- [x] 2.1 Add a shared driver API crate (source/target contracts, capability model, error taxonomy)
- [x] 2.2 Add runtime driver registry and registration wiring in Hub and Agent runtimes
- [x] 2.3 Add contract tests for registry lookup, unknown-driver handling, and capability exposure

## 3. Job spec and protocol compatibility
- [ ] 3.1 Introduce `JobSpecV2` source/target envelopes with explicit `type`, `version`, `config`, and `auth_refs`
- [ ] 3.2 Add V1 -> V2 translation layer and keep V1 API acceptance
- [ ] 3.3 Add validation rules to prevent inline credentials and enforce secret reference semantics
- [ ] 3.4 Add backward-compatible Hub-Agent protocol fields for driver identifiers and capabilities
- [ ] 3.5 Add regression tests for mixed V1/V2 jobs and mixed Hub-Agent versions

## 4. Execution planner and runtime migration
- [ ] 4.1 Implement capability-based `ExecutionPlanner` to derive execution mode from source/target/pipeline
- [ ] 4.2 Replace hard-coded source-target special branches in scheduler and agent execution with planner output
- [ ] 4.3 Ensure planner decisions are emitted to run events and summaries for observability
- [ ] 4.4 Add regression tests for planner decisions (rolling upload, direct upload, fallback paths)

## 5. Unified target lifecycle
- [ ] 5.1 Migrate backup upload path to target lifecycle API (`open_writer/upload/finalize/abort`)
- [ ] 5.2 Migrate restore and artifact stream reads to target lifecycle API (`open_reader`)
- [ ] 5.3 Migrate incomplete-run cleanup to target lifecycle API (`cleanup_run`)
- [ ] 5.4 Migrate run artifact snapshot serialization to target lifecycle API (`snapshot_redacted`)
- [ ] 5.5 Add regression tests for lifecycle correctness and idempotency across backup/restore/cleanup

## 6. Observability and quality gates
- [ ] 6.1 Standardize metrics/events dimensions by driver and plan mode
- [ ] 6.2 Add mandatory driver contract test harness and matrix test jobs in CI
- [ ] 6.3 Document developer workflow for adding a new source or target driver

## 7. Validation and rollout
- [x] 7.1 Run `bash scripts/ci.sh`
- [ ] 7.2 Run staged rollout verification in mixed-version environments
- [ ] 7.3 Mark all completed tasks and prepare milestone commits
