## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for: foundation tokens, semantic utility classes, component recipes, documentation, and guardrails
- [x] 1.2 Run `openspec validate harden-web-ui-visual-consistency --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Foundation Tokens + Utilities
- [x] 2.1 Add global foundation tokens (radii / motion / optional typography helpers) in `ui/src/styles/main.css`
- [x] 2.2 Add semantic utility classes for common UI meaning (muted text, divider/border, inset surface, code/mono blocks)
- [x] 2.3 Align Naive UI theme overrides with the foundation tokens where applicable (e.g. radii)
- [x] 2.4 Commit foundation token + utility changes (detailed message)

## 3. Web UI - Consistency Refactor (High-Leverage Areas)
- [x] 3.1 Replace non-token “visual” colors in shared layout/components (AppShell/AuthLayout/MobileTopBar/PageHeader)
- [x] 3.2 Standardize list separators + row hover/pressed states to token-driven equivalents
- [x] 3.3 Standardize “muted text” to token semantics (avoid `opacity-*` for text)
- [x] 3.4 Standardize status/error styling to use theme tokens or Naive UI semantic components (no `text-red-*`)
- [x] 3.5 Commit refactor batch(es) (detailed message; split by area if large)

## 4. Docs - UI Style Guide (EN + zh-CN)
- [ ] 4.1 Add a developer-facing UI style guide page under `docs/dev/` (tokens, scales, recipes, do/don’t)
- [ ] 4.2 Add a zh-CN version under `docs/zh/dev/`
- [ ] 4.3 Link the guide from the docs sidebars (EN + zh-CN)
- [ ] 4.4 Commit docs updates (detailed message)

## 5. Guardrails
- [ ] 5.1 Add a fast rg-based check for banned “non-token” patterns in `ui/src`
- [ ] 5.2 Integrate the check into `scripts/ci.sh`
- [ ] 5.3 Commit guardrails (detailed message)

## 6. Tests
- [ ] 6.1 Add/adjust tests for changes that are easy to regress (token usage, docs build wiring, etc.)
- [ ] 6.2 Commit tests (detailed message)

## 7. Validation
- [ ] 7.1 Run `npm test --prefix ui`
- [ ] 7.2 Run `npm run lint --prefix ui`
- [ ] 7.3 Run `npm run build-only --prefix ui`
- [ ] 7.4 Run `DOCS_BASE=/docs/ npm run build --prefix docs`
- [ ] 7.5 Run `bash scripts/ci.sh`
