## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for “最近错误” single-line truncation + full text access
- [x] 1.2 Run `openspec validate fix-incomplete-cleanup-last-error-ellipsis --strict`

## 2. UI
- [x] 2.1 Make “最近错误” column single-line ellipsis (no row height growth)
- [x] 2.2 Add hover tooltip and click-to-view-full behavior for “最近错误”
- [x] 2.3 Prevent long errors from expanding table width (avoid horizontal scroll in normal desktop widths)

## 3. Verification
- [x] 3.1 Verify visually (desktop)
- [x] 3.2 Run `cd ui && npm test` and `npm run type-check`
