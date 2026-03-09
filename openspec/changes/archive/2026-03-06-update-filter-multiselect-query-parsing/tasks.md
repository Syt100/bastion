## 1. Spec
- [x] 1.1 Add `backend` spec delta for multi-value query param key variants
- [x] 1.2 Run `openspec validate update-filter-multiselect-query-parsing --strict`

## 2. Backend
- [x] 2.1 Parse multi-value filters from raw query (`status`, `status[]`, etc.)
- [x] 2.2 Add/adjust integration tests for multi-value filtering

## 3. Verification
- [x] 3.1 Run `cargo test`
