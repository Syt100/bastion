## 1. Spec & Design
- [ ] Write and validate spec deltas under `openspec/changes/add-backup-mvp/specs/`
- [ ] Finalize manifest/entries schema and repository layout
- [ ] Define Hub/Agent JSON message schema and versioning rules

## 2. Project Scaffolding (Official Tooling)
- [ ] Create Rust workspace via `cargo new`/`cargo init`
- [ ] Create Vue 3 app via `npm create vue@latest` (TypeScript + Router + Pinia + ESLint + Vitest)
- [ ] Add Naive UI + Tailwind + ECharts to the Vue project
- [ ] Embed built UI assets into the Rust binary (single-file deployment)

## 3. Backend: Control Plane (Hub)
- [ ] HTTP server (bind `127.0.0.1:9876` by default; configurable host/port)
- [ ] Reverse-proxy-safe auth: cookie session (SQLite) + CSRF + login throttling
- [ ] Secrets store: `data/master.key`, encrypted secrets in SQLite
- [ ] Keypack export/import + key rotation workflow
- [ ] Jobs CRUD + scheduler (cron) + overlap policy (reject/queue)
- [ ] Runs/history + structured events/logs stored in SQLite (retention default 180 days, configurable)

## 4. Agent & Hub/Agent Protocol
- [ ] Enrollment token generation (TTL + usage limits)
- [ ] Agent registration to obtain `agent_id` + `agent_key` (Hub stores hash; supports revoke/rotate)
- [ ] Agent-initiated WebSocket connection + hello/capabilities
- [ ] Task dispatch + ACK/sequence + reconnect handling
- [ ] Explicit insecure mode (`--insecure-http`) with persistent UI warnings

## 5. Backup Engine
- [ ] Filesystem source (include/exclude patterns; symlinks/hardlinks handling; error policy)
- [ ] SQLite source using online backup API (no downtime) + optional `PRAGMA integrity_check`
- [ ] Packaging pipeline: tar(PAX) → zstd(level=3, threads=auto) → optional age → split parts
- [ ] Manifest v1 + entries index + atomic completion marker
- [ ] Restore flow (concatenate parts → decrypt → decompress → untar)
- [ ] Restore drill verification (download → restore to temp dir → compare hashes + SQLite integrity checks)

## 6. Targets
- [ ] WebDAV target with split-part upload, retries, and resume by existing part size
- [ ] Incomplete-run cleanup (no `complete.json` older than N days)

## 7. Notifications
- [ ] Email notifications (SMTP) with retry/backoff and dedupe per run
- [ ] WeCom group bot webhook notifications with retry/backoff and dedupe per run

## 8. Web UI
- [ ] Modern layout (sidebar/topbar), responsive + dark mode
- [ ] i18n: default `zh-CN`, support `zh-CN` + `en-US`, user switch + persistence
- [ ] Login flow + session handling
- [ ] Agents page (status, enroll, revoke)
- [ ] Jobs pages (list/create/edit/run now)
- [ ] Run history + live logs/events viewer
- [ ] Restore wizard (select run, destination, conflict strategy)
- [ ] Verify drill wizard + results

## 9. Testing & Quality
- [ ] Commit policy: commit after each milestone feature and each bug fix (see `specs/development-workflow/spec.md`)
- [ ] Rust unit tests for: manifest/entries, secrets crypto, WebDAV resume logic, scheduler policies
- [ ] Vue unit tests for: core views, forms validation, run log viewer components
- [ ] CI scripts: `cargo fmt`, `cargo clippy`, `cargo test`, `npm test`

## 10. Docs
- [ ] Reverse proxy configuration examples (Nginx/Caddy) incl. WebSocket
- [ ] Data directory layout + backup/restore of `master.key` keypack
- [ ] Vaultwarden recipe guide (Docker/Compose, required mounts)
