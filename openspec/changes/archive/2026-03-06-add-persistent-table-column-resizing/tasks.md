## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for persistent column resizing (per list)
- [x] 1.2 Run `openspec validate add-persistent-table-column-resizing --strict`

## 2. UI
- [x] 2.1 Add shared utility/composable to persist column widths (debounced writes)
- [x] 2.2 Enable resizable columns on “不完整运行清理” desktop table and persist widths
- [x] 2.3 Enable resizable columns on “通知/队列” desktop table and persist widths

## 3. Verification
- [x] 3.1 Verify in browser (resize + refresh + per-page isolation)
- [x] 3.2 Run `cd ui && npm test` and `npm run type-check`
