-- Composite indexes for keyset list paths.
--
-- Snapshots: WHERE job_id=? [AND status=?] ORDER BY ended_at DESC, run_id DESC
CREATE INDEX IF NOT EXISTS idx_run_artifacts_job_ended_run
  ON run_artifacts(job_id, ended_at DESC, run_id DESC);

CREATE INDEX IF NOT EXISTS idx_run_artifacts_job_status_ended_run
  ON run_artifacts(job_id, status, ended_at DESC, run_id DESC);

-- Notifications queue: optional status/channel filters + keyset ORDER BY created_at DESC, id DESC
CREATE INDEX IF NOT EXISTS idx_notifications_created_id
  ON notifications(created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_notifications_status_channel_created_id
  ON notifications(status, channel, created_at DESC, id DESC);
