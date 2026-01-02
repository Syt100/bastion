## 1. Spec
- [x] 1.1 Add spec deltas for notifications management (channels/destinations/templates/queue + semantics)
- [x] 1.2 Add spec deltas for Web UI settings submenu + notifications pages (desktop + mobile)
- [x] 1.3 Add spec deltas for per-job notifications override in job spec + UI wizard
- [x] 1.4 Run `openspec validate update-notifications-management --strict`

## 2. Storage (SQLite)
- [x] 2.1 Add migrations for settings + notification destination metadata
- [x] 2.2 Add repos for settings + notification destinations
- [x] 2.3 Extend notifications repo for queue listing + retry/cancel operations
- [x] 2.4 Add/extend unit tests for repos

## 3. Backend HTTP API
- [x] 3.1 Add endpoints for notification settings (global/channels/templates)
- [x] 3.2 Add endpoints for notification destinations (list + enable/disable + test send)
- [x] 3.3 Add endpoints for notification queue (paged list + retry-now/cancel)
- [x] 3.4 Ensure queue cancellation semantics when deleting destinations or disabling channels
- [x] 3.5 Add/extend unit tests for APIs/handlers where appropriate

## 4. Engine
- [x] 4.1 Enqueue notifications based on global settings + per-job overrides + destination enabled state
- [x] 4.2 Worker respects settings/destination state and marks canceled when disabled/missing
- [x] 4.3 Implement global templates rendering for email/wecom messages
- [x] 4.4 Add/extend unit tests for selection/templating logic where appropriate

## 5. Web UI
- [x] 5.1 Refactor `/settings` into a submenu shell with route-based child pages
- [x] 5.2 Add Notifications pages: Channels, Destinations, Templates, Queue (desktop tabs + mobile selector)
- [x] 5.3 Add per-job notifications config to job create/edit wizard (inherit/custom + channel/destinations)
- [x] 5.4 Add/extend unit tests (Vitest) for key flows
- [x] 5.5 Ensure i18n coverage for new UI strings (zh-CN default + en-US)

## 6. Validation
- [x] 6.1 Run `cargo test`
- [x] 6.2 Run `cargo clippy --all-targets --all-features -D warnings`
- [x] 6.3 Run `cargo fmt --check`
- [x] 6.4 Run `npm test` (ui)
- [x] 6.5 Run `npm run build` (ui)

## 7. Commits
- [x] 7.1 Commit the spec proposal (detailed message)
- [x] 7.2 Commit backend changes (detailed message with Modules/Tests)
- [x] 7.3 Commit engine changes (detailed message with Modules/Tests)
- [x] 7.4 Commit Web UI changes (detailed message with Modules/Tests)
