-- Add cancel-request metadata and terminal canceled status support.
ALTER TABLE runs ADD COLUMN cancel_requested_at INTEGER;
ALTER TABLE runs ADD COLUMN cancel_requested_by_user_id INTEGER;
ALTER TABLE runs ADD COLUMN cancel_reason TEXT;

ALTER TABLE operations ADD COLUMN cancel_requested_at INTEGER;
ALTER TABLE operations ADD COLUMN cancel_requested_by_user_id INTEGER;
ALTER TABLE operations ADD COLUMN cancel_reason TEXT;

CREATE INDEX IF NOT EXISTS idx_runs_cancel_requested_at
  ON runs(cancel_requested_at);

CREATE INDEX IF NOT EXISTS idx_operations_cancel_requested_at
  ON operations(cancel_requested_at);
