---
## 1. Spec
- [x] 1.1 Draft proposal, tasks, design, and spec deltas (`control-plane`, `hub-agent-protocol`, `backend`, `web-ui`, `targets-webdav`)
- [x] 1.2 Run `openspec validate add-webdav-browse-api --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Targets - WebDAV client PROPFIND support
- [ ] 2.1 Add WebDAV directory listing helper (PROPFIND depth=1) and parse common properties
- [ ] 2.2 Add unit tests for XML parsing and edge cases
- [ ] 2.3 Commit targets-webdav changes (detailed message)

## 3. Control-plane - WebDAV list endpoint
- [ ] 3.1 Add `GET/POST /api/nodes/{node_id}/webdav/list` endpoint and request/response types
- [ ] 3.2 Implement hub execution and agent forwarding (node-scoped)
- [ ] 3.3 Map errors to stable picker-friendly error codes
- [ ] 3.4 Commit control-plane changes (detailed message)

## 4. Hub↔Agent protocol forwarding
- [ ] 4.1 Add Hub↔Agent message types for WebDAV list requests/results
- [ ] 4.2 Implement Agent handler that performs PROPFIND using its local secrets snapshot
- [ ] 4.3 Commit hub/agent changes (detailed message)

## 5. Web UI - Picker integration
- [ ] 5.1 Implement WebDAV picker data source using the new API
- [ ] 5.2 Add “Browse” for WebDAV prefix selection in restore wizard (mobile-friendly)
- [ ] 5.3 Add i18n strings and validation messages
- [ ] 5.4 Commit UI changes (detailed message)

## 6. Verification
- [ ] 6.1 Run `cargo test --workspace`
- [ ] 6.2 Run `npm test --prefix ui`
- [ ] 6.3 Run `npm run type-check --prefix ui`
