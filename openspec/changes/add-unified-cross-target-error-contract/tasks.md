## 1. Spec
- [x] 1.1 Finalize backend and web-ui spec deltas for unified error envelope and protocol extensions
- [x] 1.2 Run `openspec validate add-unified-cross-target-error-contract --strict`

## 2. Design
- [x] 2.1 Define canonical error envelope fields, required/optional semantics, and schema versioning
- [x] 2.2 Define protocol extension mapping (`http`, `sftp`, `drive_api`, `file`) and async/partial-failure structures
- [x] 2.3 Define compatibility policy for old fields and rollout phases

## 3. Backend implementation
- [x] 3.1 Introduce shared error-envelope types and helper builders in core/engine path
- [x] 3.2 Migrate high-frequency failure emitters (run failed, agent dispatch, notification failure, cleanup/artifact-delete)
- [x] 3.3 Keep legacy fields during transition and attach envelope in stable field location
- [x] 3.4 Add regression tests for protocol-agnostic fields, retriable semantics, and fallback behavior

## 4. Web UI implementation
- [x] 4.1 Render envelope-based diagnostics (message/hint keys + params) with locale fallback
- [x] 4.2 Add protocol-specific detail rows (for example HTTP status vs SFTP provider code)
- [x] 4.3 Surface async-operation and partial-failure diagnostics in detail panels
- [x] 4.4 Add/adjust UI tests for localization and envelope fallback behavior

## 5. Validation and release notes
- [x] 5.1 Run targeted Rust and UI tests for updated modules
- [x] 5.2 Run `scripts/ci.sh`
- [x] 5.3 Update `CHANGELOG.md` if user-visible diagnostics change in this rollout
