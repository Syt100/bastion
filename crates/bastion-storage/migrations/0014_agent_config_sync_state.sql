-- Agent config sync observability: desired vs applied snapshots and error summaries.
ALTER TABLE agents ADD COLUMN desired_config_snapshot_id TEXT;
ALTER TABLE agents ADD COLUMN desired_config_snapshot_at INTEGER;

ALTER TABLE agents ADD COLUMN applied_config_snapshot_id TEXT;
ALTER TABLE agents ADD COLUMN applied_config_snapshot_at INTEGER;

ALTER TABLE agents ADD COLUMN last_config_sync_attempt_at INTEGER;
ALTER TABLE agents ADD COLUMN last_config_sync_error_kind TEXT;
ALTER TABLE agents ADD COLUMN last_config_sync_error TEXT;
ALTER TABLE agents ADD COLUMN last_config_sync_error_at INTEGER;

