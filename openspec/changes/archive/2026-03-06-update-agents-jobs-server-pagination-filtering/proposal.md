# Change: Upgrade agents/jobs list to server-side pagination and filtering

## Why
Current agents/jobs list pages still perform core filtering and pagination on the client after downloading full result sets. This creates unnecessary payload and render pressure at larger scales, and duplicates filtering logic between frontend and backend.

## What Changes
- Extend `/api/agents` with server-side search/status filtering and page/page_size pagination while preserving existing label filters.
- Extend `/api/jobs` with server-side node scope/search/status/schedule filtering, sort, and page/page_size pagination.
- Return unified list payloads for both endpoints: `items`, `page`, `page_size`, `total`.
- Update UI stores and views (`AgentsView`, `JobsWorkspaceShellView`) to drive pagination/filter state through backend query params instead of client slicing/filtering.
- Keep selection/bulk-action behavior stable under remote list paging.
- Add/adjust backend and UI tests for query semantics, pagination metadata, and remote list rendering.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected areas (representative):
  - `crates/bastion-http/src/http/agents/admin.rs`
  - `crates/bastion-http/src/http/jobs/crud.rs`
  - `crates/bastion-http/src/http/filter_multiselect_tests.rs`
  - `crates/bastion-http/src/http/jobs_list_tests.rs`
  - `ui/src/stores/agents.ts`
  - `ui/src/stores/jobs.ts`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`

## Non-Goals
- No dependency vulnerability governance changes.
- No changes to authentication/session or CSRF flows.
- No migration of unrelated list pages outside agents/jobs.
