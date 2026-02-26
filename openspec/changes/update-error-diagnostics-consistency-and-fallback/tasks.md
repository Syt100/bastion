## 1. Spec
- [x] 1.1 Add `backend` and `web-ui` spec deltas for diagnostics consistency and fallback guidance
- [x] 1.2 Run `openspec validate update-error-diagnostics-consistency-and-fallback --strict`

## 2. Backend - diagnostics consistency
- [x] 2.1 Standardize WebDAV `get_bytes`/`delete`/`get_to_file` failures to retain HTTP status/body/retry context
- [x] 2.2 Add shared error-kind mapping in driver bridge so auth/config/network classes remain accurate
- [x] 2.3 Extend run failed fallback classifier with storage-capacity detection and non-WebDAV-biased unknown hint
- [x] 2.4 Add `hint` fields for incomplete-cleanup and artifact-delete failure/block/abandon events
- [x] 2.5 Add regression tests for classifier mapping and fallback hint behavior

## 3. Web UI - diagnostics presentation
- [x] 3.1 Localize run-event detail hint label in both desktop modal and mobile drawer
- [x] 3.2 Add/update UI tests for localized hint label rendering

## 4. Validation
- [x] 4.1 Run targeted Rust and UI test suites for changed modules
- [x] 4.2 Run `scripts/ci.sh`

## 5. Release notes
- [x] 5.1 Update `CHANGELOG.md` via `maintain-changelog-release`
