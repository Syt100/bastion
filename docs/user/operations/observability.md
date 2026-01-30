# Observability (health, readiness, and request IDs)

For small deployments, the most useful observability is:

- liveness/readiness probes
- request IDs to correlate UI errors with server logs
- clear log locations (console/journald/file)

## Health endpoints

These endpoints return a small JSON document with an `ok` boolean.

### Liveness: `GET /api/health`

Use this to answer: “is the process running?”

- Expected: `200` with `{ "ok": true }`

### Readiness: `GET /api/ready`

Use this to answer: “can the Hub serve requests?”

This readiness probe checks basic dependencies (including database connectivity).

- Expected: `200` with `{ "ok": true }`
- If dependencies are not ready: `503` with `{ "ok": false }`

### System info: `GET /api/system`

This endpoint returns basic runtime/build info, for example:

- version (tag or crate version)
- build time (if available)
- whether `--insecure-http` is enabled
- configured Hub timezone

## Request IDs

Bastion assigns an `x-request-id` header to HTTP responses.

When reporting a bug, include:

- the request URL
- the `x-request-id` value
- relevant log lines around that request ID

This makes it much easier to correlate client-side errors with server logs.

Tip: the Web UI shows a request ID in error dialogs; include it in bug reports.

## Logs

See: [Logging](/user/operations/logging).

Quick hints:

- default: logs to console
- systemd: logs are captured by journald (`journalctl -u bastion -f`)
- Windows Service: default log file under `%PROGRAMDATA%\\bastion\\logs\\bastion.log` (unless configured)
