-- Persist the latest progress snapshot for long-running work.
ALTER TABLE runs ADD COLUMN progress_json TEXT;
ALTER TABLE operations ADD COLUMN progress_json TEXT;

