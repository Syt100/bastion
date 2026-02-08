## 1. Spec
- [x] 1.1 Draft proposal and task checklist for server-side list pagination/filtering
- [x] 1.2 Add backend spec delta for agents/jobs list query/response semantics
- [x] 1.3 Add web-ui spec delta for remote list state integration
- [x] 1.4 Run `openspec validate update-agents-jobs-server-pagination-filtering --strict`

## 2. Backend implementation
- [x] 2.1 Extend `/api/agents` list handler with `status`, `q`, `page`, `page_size` query params and paged response metadata
- [x] 2.2 Extend `/api/jobs` list handler with node scope/search/status/schedule/sort + pagination and total count
- [x] 2.3 Add/update HTTP tests covering pagination, filtering, and total/page metadata for agents/jobs list APIs

## 3. Frontend implementation
- [x] 3.1 Update agents/jobs stores to consume paged list responses and pass remote filter/pagination params
- [x] 3.2 Refactor `AgentsView` to remote filtering/pagination (remove client-side slicing/filtering)
- [x] 3.3 Refactor `JobsWorkspaceShellView` to remote filtering/sort/pagination while preserving bulk/select behavior
- [x] 3.4 Update/extend related UI unit tests for remote list mode

## 4. Validation
- [x] 4.1 Run `npm --prefix ui run lint:check`
- [x] 4.2 Run `npm --prefix ui run type-check`
- [x] 4.3 Run `npm --prefix ui run test -- --run`
- [x] 4.4 Run `bash scripts/ci.sh`
