## 1. Spec & Design
- [x] Write and validate spec deltas under `openspec/changes/add-backup-mvp/specs/`
- [x] Finalize manifest/entries schema and repository layout
- [x] Define Hub/Agent JSON message schema and versioning rules

## 2. Project Scaffolding (Official Tooling)
- [x] Create Rust workspace via `cargo new`/`cargo init`
- [x] Create Vue 3 app via `npm create vue@latest` (TypeScript + Router + Pinia + ESLint + Vitest)
- [x] Add Naive UI + Tailwind + ECharts to the Vue project
- [x] Embed built UI assets into the Rust binary (single-file deployment)

## 3. Backend: Control Plane (Hub)
- [x] HTTP server (bind `127.0.0.1:9876` by default; configurable host/port)
- [x] Public access auth: reverse-proxy-safe cookie sessions + CSRF + login throttling + HTTPS enforcement
- [x] Secrets store: `data/master.key`, encrypted secrets in SQLite
- [x] Keypack export/import + key rotation workflow
- [x] Jobs CRUD + scheduler (cron) + overlap policy (reject/queue)
- [x] Runs/history + structured events/logs stored in SQLite (retention default 180 days, configurable)

## 4. Agent & Hub/Agent Protocol
- [x] Enrollment token generation (TTL + usage limits)
- [x] Agent registration to obtain `agent_id` + `agent_key` (Hub stores hash; supports revoke/rotate)
- [x] Agent-initiated WebSocket connection + hello/capabilities
- [x] Task dispatch + ACK/sequence + reconnect handling
- [x] Explicit insecure mode (`--insecure-http`) with persistent UI warnings

## 5. Backup Engine
- [x] Filesystem source (include/exclude patterns; symlinks/hardlinks handling; error policy)
- [x] SQLite source using online backup API (no downtime) + optional `PRAGMA integrity_check`
- [x] Packaging pipeline: tar(PAX) → zstd(level=3, threads=auto) → optional age → split parts
- [x] Manifest v1 + entries index + atomic completion marker
- [x] Restore flow (concatenate parts → decrypt → decompress → untar)
- [x] Restore drill verification (download → restore to temp dir → compare hashes + SQLite integrity checks)

## 6. Targets
- [x] WebDAV target with split-part upload, retries, and resume by existing part size
- [x] Local directory target (store runs under `<base_dir>/<job_id>/<run_id>/`)
- [ ] Incomplete-run cleanup (no `complete.json` older than N days)

## 7. Notifications
- [ ] Email notifications (SMTP) with retry/backoff and dedupe per run
- [x] WeCom group bot webhook notifications with retry/backoff and dedupe per run

## 8. Web UI
- [x] Modern layout (sidebar/topbar), responsive + dark mode
- [x] i18n: default `zh-CN`, support `zh-CN` + `en-US`, user switch + persistence
- [x] Login flow + session handling
- [x] Agents page (status, enroll, revoke)
- [x] Jobs pages (list/create/edit/run now)
- [x] Run history + live logs/events viewer
- [x] Restore wizard (select run, destination, conflict strategy)
- [x] Verify drill wizard + results

## 9. Testing & Quality
- [x] Commit policy: commit after each milestone feature and each bug fix (see `specs/development-workflow/spec.md`)
- [ ] Rust unit tests for: manifest/entries, secrets crypto, WebDAV resume logic, scheduler policies
- [ ] Vue unit tests for: core views, forms validation, run log viewer components
- [ ] CI scripts: `cargo fmt`, `cargo clippy`, `cargo test`, `npm test`

## 10. Docs
- [ ] Reverse proxy configuration examples (Nginx/Caddy) incl. WebSocket
- [ ] Data directory layout + backup/restore of `master.key` keypack
- [ ] Vaultwarden recipe guide (Docker/Compose, required mounts)
