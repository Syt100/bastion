---
## 1. Spec
- [x] 1.1 Draft proposal, tasks, design, and spec deltas (`control-plane`, `hub-agent`, `hub-agent-protocol`, `backend`, `web-ui`)
- [x] 1.2 Run `openspec validate add-restore-destinations-and-executors --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Control-plane - Restore API model
- [x] 2.1 Update `POST /api/runs/{run_id}/restore` request to accept a typed `destination` and optional `executor`
- [x] 2.2 Validate destination fields (local_fs path; webdav base_url/secret_name/prefix; conflict policy; selection non-empty)
- [x] 2.3 Persist restore request summary into operation events
- [x] 2.4 Commit control-plane changes (detailed message)

## 3. Hub↔Agent - Restore execution + relay
- [x] 3.1 Add Hub↔Agent protocol messages for restore tasks and operation events/results
- [x] 3.2 Implement Hub-side relay for artifact streaming across nodes (Hub as intermediary)
- [x] 3.3 Implement Agent-side restore task handler + artifact stream client
- [x] 3.4 Ensure reconnect-safe behavior: task persistence + idempotency + retries
- [x] 3.5 Commit hub/agent changes (detailed message)

## 4. Restore destinations
- [x] 4.1 Implement `local_fs` destination on Hub (existing behavior via sink)
- [x] 4.2 Implement `local_fs` destination on Agent (executor=agent)
- [x] 4.3 Implement `webdav` destination sink with prefix support
- [x] 4.4 Implement `.bastion-meta/` sidecar write for WebDAV destinations (per op_id)
- [x] 4.5 Commit restore destination changes (detailed message)

## 5. Web UI - Restore wizard updates
- [x] 5.1 Extend restore wizard to select destination type and node (for local_fs) with mobile-friendly layout
- [x] 5.2 Add WebDAV destination form (base_url, secret, prefix input) with `.bastion-meta` note
- [x] 5.3 Wire request payload to updated restore API
- [x] 5.4 Update i18n strings and validation messages
- [x] 5.5 Commit UI changes (detailed message)

## 6. Verification
- [x] 6.1 Run `cargo test --workspace`
- [x] 6.2 Run `npm test --prefix ui`
- [x] 6.3 Run `npm run type-check --prefix ui`
