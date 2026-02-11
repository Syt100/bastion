## 1. Spec
- [x] 1.1 Create proposal/tasks/spec deltas for final restore reader decoupling
- [x] 1.2 Run `openspec validate refactor-restore-target-reader-final-decoupling --strict`

## 2. Driver reader contract
- [x] 2.1 Extend `bastion-driver-api` target contract with reader trait and `open_reader`
- [x] 2.2 Move concrete reader construction into built-in target drivers
- [x] 2.3 Make registry `open_reader` delegate directly to target drivers

## 3. Runtime integration
- [x] 3.1 Add shared target runtime config resolver helpers in `bastion-driver-registry`
- [x] 3.2 Migrate hub/agent store + planner + snapshot to shared helpers
- [x] 3.3 Migrate restore access/entries fetch + restore source adapter to reader trait
- [x] 3.4 Migrate hub artifact stream read path to reader trait (preserving local-agent fast path)

## 4. Quality and docs
- [x] 4.1 Add/adjust contract regression tests for reader and target config resolver
- [x] 4.2 Run `bash scripts/ci.sh`
- [x] 4.3 Sync EN/ZH driver platform docs and mark all tasks done
