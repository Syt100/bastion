## 1. Spec
- [x] 1.1 Draft proposal, tasks, and spec deltas (no omissions)
- [x] 1.2 Run `openspec validate add-fs-list-pagination --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - HTTP FS List Pagination
- [x] 2.1 Extend `/api/nodes/{node_id}/fs/list` query params (cursor/limit + filters)
- [x] 2.2 Return `next_cursor` (and optional `total`) in the response
- [x] 2.3 Ensure Hub local listing is memory-safe for huge directories (bounded memory per request)
- [x] 2.4 Add/adjust backend tests (as available) and verify error mapping unchanged
- [x] 2.5 Commit backend HTTP changes (detailed message)

## 3. Backend - Agent Protocol FS List Pagination
- [x] 3.1 Extend `HubToAgentMessageV1::FsList` and `AgentToHubMessageV1::FsListResult` with optional paging/filter fields
- [x] 3.2 Update Agent handler to implement paged listing (bounded memory per request)
- [x] 3.3 Update Hub agent_manager to request pages and surface `next_cursor`/`total`
- [x] 3.4 Commit protocol + agent/hub changes (detailed message)

## 4. UI - Filesystem Picker
- [x] 4.1 Update FsPathPickerModal to fetch paged results and render “加载更多”
- [x] 4.2 Move filtering semantics to server-side fetch (refresh list when filters change)
- [x] 4.3 Add UI tests for paging/filter fetch behavior
- [x] 4.4 Commit UI changes (detailed message)

## 5. Verification
- [x] 5.1 Run `cargo test` (or targeted tests if full suite is too slow)
- [x] 5.2 Run `npm test --prefix ui`
- [x] 5.3 Run `npm run type-check --prefix ui`
- [x] 5.4 Verify manually in browser (desktop + mobile)
