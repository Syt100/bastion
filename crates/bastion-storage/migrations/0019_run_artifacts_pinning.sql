-- Snapshot pinning ("backup data management").
--
-- Pinned snapshots are protected from automated retention and require an explicit
-- force flag for manual deletion.

ALTER TABLE run_artifacts ADD COLUMN pinned_at INTEGER;
ALTER TABLE run_artifacts ADD COLUMN pinned_by_user_id INTEGER;

CREATE INDEX IF NOT EXISTS idx_run_artifacts_pinned_at ON run_artifacts(pinned_at);
