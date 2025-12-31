# Logging

## Defaults
- Bastion logs to the console by default.
- If no explicit filter is configured, Bastion uses a conservative default filter:
  - `info,tower_http=warn`
  - This keeps Bastion `INFO` logs visible while suppressing noisy per-request HTTP access logs.

## Log Filter (verbosity)
You can configure the log filter using one of:
- `--log "<filter>"`
- `BASTION_LOG="<filter>"`
- `RUST_LOG="<filter>"`

The filter syntax follows the standard `tracing_subscriber::EnvFilter` format.

Examples:
```bash
# Default (INFO)
./bastion

# More details from Bastion code, keep HTTP logs quiet
./bastion --log "bastion=debug,tower_http=warn"

# Enable HTTP request logs too
./bastion --log "info,tower_http=info"
```

## File Logging + Rotation
Enable file logging (in addition to the console) with:
- `--log-file /path/to/bastion.log`
- or `BASTION_LOG_FILE=/path/to/bastion.log`

Rotation options:
- `--log-rotation daily|hourly|never` (default: `daily`)
- `BASTION_LOG_ROTATION=daily|hourly|never`

Retention (rotated logs only):
- `--log-keep-files <N>` (default: `30`, `0` disables pruning)
- `BASTION_LOG_KEEP_FILES=<N>`

Notes:
- Rotated files use the configured file name as a prefix:
  - daily: `bastion.log.YYYY-MM-DD`
  - hourly: `bastion.log.YYYY-MM-DD-HH`
- Bastion prunes only files that match these rotation name patterns.

Example:
```bash
./bastion \
  --log-file ./data/logs/bastion.log \
  --log-rotation daily \
  --log-keep-files 30
```

## Secret Redaction
Bastion MUST NOT log secret material (passwords, tokens, private keys).
If you include credentials directly in a URL, Bastion attempts to redact them in logs, but you should still avoid embedding secrets in URLs.
