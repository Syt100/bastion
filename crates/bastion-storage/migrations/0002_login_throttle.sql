-- Login throttling to mitigate brute-force attacks.
CREATE TABLE IF NOT EXISTS login_throttle (
  ip TEXT PRIMARY KEY,
  failures INTEGER NOT NULL,
  first_failed_at INTEGER NOT NULL,
  last_failed_at INTEGER NOT NULL,
  locked_until INTEGER
);

CREATE INDEX IF NOT EXISTS idx_login_throttle_locked_until ON login_throttle(locked_until);
CREATE INDEX IF NOT EXISTS idx_login_throttle_last_failed_at ON login_throttle(last_failed_at);

