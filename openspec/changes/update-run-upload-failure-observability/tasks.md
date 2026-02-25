## 1. Spec
- [x] 1.1 Add backend spec delta for rolling uploader error propagation, structured diagnostics, and WebDAV tuning controls
- [x] 1.2 Add web-ui spec delta for actionable run failure rendering
- [x] 1.3 Run `openspec validate update-run-upload-failure-observability --strict`

## 2. Backend implementation
- [x] 2.1 Refactor rolling archive bridge to preserve uploader root cause when sender observes receiver drop
- [x] 2.2 Ensure execute paths always reconcile uploader and build outcomes, with root-cause-first error selection
- [x] 2.3 Preserve error chains in archive write failures (avoid string-only wrapping that drops source diagnostics)
- [x] 2.4 Add run failure diagnostics extraction and attach structured fields to final `failed` run event
- [x] 2.5 Implement WebDAV upload error classifier + operator hint mapping and include per-part context

## 3. WebDAV tuning controls
- [x] 3.1 Extend WebDAV request limit schema with timeout/retry knobs and defaults
- [x] 3.2 Wire new limits through engine/driver/target conversions and validation
- [x] 3.3 Apply limits in WebDAV client request construction and retry behavior

## 4. Web UI
- [x] 4.1 Enhance run events chips/details to highlight failure hints and key transport metadata
- [x] 4.2 Add/update UI tests for diagnostic chip extraction and rendering behavior

## 5. Tests and validation
- [x] 5.1 Add backend regression tests for rolling uploader drop root-cause propagation
- [x] 5.2 Add backend tests for failure field classification (HTTP 413/timeout/auth/rate-limit)
- [x] 5.3 Run targeted tests for changed crates
- [x] 5.4 Run `bash scripts/ci.sh`

## 6. Release notes
- [x] 6.1 Update `CHANGELOG.md` via `maintain-changelog-release`
