## 1. Spec
- [x] 1.1 Draft proposal/tasks/spec delta for dashboard first-screen performance + loading UX
- [x] 1.2 Run `openspec validate update-dashboard-first-screen-performance-and-loading --strict`

## 2. Implementation
- [x] 2.1 Extract desktop recent-runs table into async-loaded component to reduce dashboard route-path payload
- [x] 2.2 Introduce shared viewport-lazy helper and apply it to dashboard trend/recent sections
- [x] 2.3 Add lightweight loading animation component and use it in deferred dashboard section placeholders
- [x] 2.4 Improve dashboard refresh button UX (`loading` state feedback)
- [x] 2.5 Add/adjust regression tests for viewport-lazy helper behavior

## 3. Validation
- [x] 3.1 Run `npm --prefix ui run lint:check`
- [x] 3.2 Run `npm --prefix ui run type-check`
- [x] 3.3 Run `npm --prefix ui run test -- --run`
- [x] 3.4 Run `bash scripts/ci.sh`
