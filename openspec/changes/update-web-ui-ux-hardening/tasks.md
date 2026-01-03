## 1. Spec
- [ ] 1.1 Add `web-ui` spec delta for: document titles/theme-color, focus-visible + reduced-motion, empty-state pattern, and abortable latest-request behavior
- [ ] 1.2 Run `openspec validate update-web-ui-ux-hardening --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Browser Chrome Details
- [ ] 2.1 Add localized route-driven `document.title` behavior
- [ ] 2.2 Update `meta[name="theme-color"]` based on light/dark mode
- [ ] 2.3 Commit browser chrome improvements (detailed message)

## 3. Web UI - Accessibility & Motion
- [ ] 3.1 Add global `:focus-visible` outline styling
- [ ] 3.2 Respect `prefers-reduced-motion` for pulse/transition-heavy effects
- [ ] 3.3 Commit a11y focus/motion improvements (detailed message)

## 4. Web UI - Empty/Loading States
- [ ] 4.1 Add shared empty-state component/pattern
- [ ] 4.2 Apply empty-state to key list pages where it improves clarity
- [ ] 4.3 Commit empty/loading state improvements (detailed message)

## 5. Web UI - Request Cancellation
- [ ] 5.1 Add a shared helper for “latest request wins” with `AbortController`
- [ ] 5.2 Apply it to rapid-refresh pages (notification queue filter/page changes)
- [ ] 5.3 Commit request cancellation hardening (detailed message)

## 6. Validation
- [ ] 6.1 Run `npm run lint --prefix ui`
- [ ] 6.2 Run `npm test --prefix ui`
- [ ] 6.3 Run `npm run build --prefix ui`

