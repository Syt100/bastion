## 1. Spec & Design
- [ ] Define which snapshot statuses should block run pruning (`present`, `deleting`, `error`)
- [ ] Validate this change with `openspec validate update-run-retention-for-snapshots --strict`

## 2. Storage Query Changes
- [ ] Update `runs_repo::prune_runs_ended_before` to skip runs that still have a non-deleted snapshot record
- [ ] Ensure the query remains efficient (indexes on `run_artifacts`)
- [ ] Add storage tests:
  - run with `run_artifacts.status=present` is not deleted
  - run with `run_artifacts.status=deleted/missing` can be deleted
  - run without snapshot can be deleted

## 3. Engine Retention Loop
- [ ] Keep the scheduler loop unchanged, but ensure it uses the updated pruning function
- [ ] Add a regression test around the loop if appropriate

