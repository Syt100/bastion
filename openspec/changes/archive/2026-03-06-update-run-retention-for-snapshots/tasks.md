## 1. Spec & Design
- [x] Define which snapshot statuses should block run pruning (`present`, `deleting`, `error`)
- [x] Validate this change with `openspec validate update-run-retention-for-snapshots --strict`

## 2. Storage Query Changes
- [x] Update `runs_repo::prune_runs_ended_before` to skip runs that still have a non-deleted snapshot record
- [x] Ensure the query remains efficient (indexes on `run_artifacts`)
- [x] Add storage tests:
  - run with `run_artifacts.status=present` is not deleted
  - run with `run_artifacts.status=deleted/missing` can be deleted
  - run without snapshot can be deleted

## 3. Engine Retention Loop
- [x] Keep the scheduler loop unchanged, but ensure it uses the updated pruning function
- [x] Add a regression test around the loop if appropriate (covered by storage-level pruning tests; loop is a thin wrapper)
