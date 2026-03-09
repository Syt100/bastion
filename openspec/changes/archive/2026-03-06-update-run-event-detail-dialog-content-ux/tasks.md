## 1. Spec
- [x] 1.1 Define web-ui requirement deltas for progressive event-detail rendering
- [x] 1.2 Run `openspec validate update-run-event-detail-dialog-content-ux --strict`

## 2. Implementation
- [x] 2.1 Update shared event-detail renderer with progressive disclosure (error chain and raw payload)
- [x] 2.2 Add concise context rendering and keep envelope diagnostics/fallback behavior intact
- [x] 2.3 Update locale strings for new actions/sections
- [x] 2.4 Add/adjust UI regression tests for the optimized dialog content

## 3. Validation
- [x] 3.1 Run targeted UI tests for run-event detail entry points
- [x] 3.2 Run `npm --prefix ui run type-check`
- [x] 3.3 Update `CHANGELOG.md` for user-visible dialog content improvements
