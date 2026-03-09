## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for: document titles/theme-color, focus-visible + reduced-motion, empty-state pattern, and abortable latest-request behavior
- [x] 1.2 Run `openspec validate update-web-ui-ux-hardening --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Browser Chrome Details
- [x] 2.1 Add localized route-driven `document.title` behavior
- [x] 2.2 Update `meta[name="theme-color"]` based on light/dark mode
- [x] 2.3 Commit browser chrome improvements (detailed message)

## 3. Web UI - Accessibility & Motion
- [x] 3.1 Add global `:focus-visible` outline styling
- [x] 3.2 Respect `prefers-reduced-motion` for pulse/transition-heavy effects
- [x] 3.3 Commit a11y focus/motion improvements (detailed message)

## 4. Web UI - Empty/Loading States
- [x] 4.1 Add shared empty-state component/pattern
- [x] 4.2 Apply empty-state to key list pages where it improves clarity
- [x] 4.3 Commit empty/loading state improvements (detailed message)

## 5. Web UI - Request Cancellation
- [x] 5.1 Add a shared helper for “latest request wins” with `AbortController`
- [x] 5.2 Apply it to rapid-refresh pages (notification queue filter/page changes)
- [x] 5.3 Commit request cancellation hardening (detailed message)

## 6. Validation
- [x] 6.1 Run `npm run lint --prefix ui`
- [x] 6.2 Run `npm test --prefix ui`
- [x] 6.3 Run `npm run build --prefix ui`
