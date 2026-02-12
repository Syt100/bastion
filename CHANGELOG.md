# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project follows [Semantic Versioning](https://semver.org/) while in pre-1.0 development.

## [Unreleased]

### Added
- Added a guided Windows MSI installer flow (license + install directory dialogs) instead of progress-only UX.
- Added Windows Start Menu shortcuts for `Bastion` and `Bastion Web UI`.
- Added an uninstall-time checkbox option to remove `C:\ProgramData\bastion` data (default remains keep data).
- Added an install-completion checkbox (default enabled) to start Bastion and open Web UI after installation.

### Changed
- Changed release `workflow_dispatch` artifact uploads to per-file granularity so Linux GNU `.tar.gz`, `.deb`, and `.rpm` outputs are downloaded separately (matching published release assets).
- Changed release `workflow_dispatch` version labeling to `<latest-tag>-dh<short-hash>` for build metadata and artifact naming, while keeping tag-triggered release labels unchanged.
- Changed Windows x86_64 MSI packaging to explicit 64-bit mode so installs target `Program Files` instead of `Program Files (x86)` on 64-bit hosts.
- Changed Windows installer metadata to include support/about/update links in Apps & features.

### Deprecated
- _No user-facing changes yet._

### Removed
- _No user-facing changes yet._

### Fixed
- Fixed Windows MSI packaging to embed installer CAB payloads so release MSI artifacts contain the expected application payload.
- Added release workflow sanity checks to fail Windows packaging if the generated MSI is unexpectedly tiny.
- Fixed Windows MSI build failure caused by invalid mixed `ComponentGroup/@Directory` and per-component `Directory` usage in WiX authoring.
- Fixed Windows MSI ICE validation failures for Start Menu shortcuts by using a per-user (`HKCU`) shortcut component key path.
- Fixed interactive uninstall flow routing so the data-retention choice dialog appears before remove confirmation across remove entry paths.
- Fixed Windows MSI packaging failure caused by scheduling uninstall prompt before a non-existent `VerifyReadyDlg` UI sequence action.

### Security
- _No user-facing changes yet._

## [v0.2.0] - 2026-02-12

### Added
- Added an authenticated About page showing Hub/UI version and build time.
- Added Hub runtime config persistence/API and a Settings page to inspect effective runtime config.
- Added Agent labels (tagging and label-based filtering).
- Added a bulk operations framework with async execution and retry support.
- Added bulk WebDAV credential distribution to selected Agents.
- Added bulk Job deploy to many nodes.
- Added Agent config-sync observability and manual sync actions.
- Added server-side pagination for large filesystem picker directories.
- Added server-side sorting for filesystem lists (name/mtime/size).
- Added breadcrumb navigation in picker path bars.
- Added picker keyboard shortcuts and accessibility labels.
- Added picker shift-range selection helpers.
- Added picker state persistence per node.
- Added restore destinations/executors (Hub/Agent execution model).
- Added Agent-executed restore via Hub relay streams.
- Added WebDAV restore destination support with `.bastion-meta` handling.
- Added WebDAV browse support end-to-end (PROPFIND target driver, hub-agent protocol, HTTP API, and UI picker).
- Added `raw_tree_v1` artifact format backup and restore paths.
- Added restore wizard artifact format selector and `raw_tree` encryption guardrails.
- Added operation-to-run subject linking for restore/verify operations.
- Added run detail API (`GET /api/runs/{id}`) and node-scoped Run Detail view.
- Added persisted run/operation progress snapshots and filesystem pre-scan progress.
- Added Run Detail stage timeline and richer progress metrics.
- Added Run Events tools: filters, search, jump, and export.
- Added snapshots index and snapshots management page.
- Added async snapshot delete queue (tasks/events) with UI controls.
- Added Agent-local snapshot delete via Agent protocol.
- Added snapshot pinning and force-delete guardrails.
- Added job-level retention policy configuration.
- Added Hub default retention and inheritance for new jobs.
- Added retention preview/apply APIs and UI editor.
- Added retention enforcement loop for snapshot cleanup.
- Added optional archive-with-snapshot-cascade behavior for jobs.
- Added source consistency tracking and warning surfaces.
- Added source consistency fingerprint report v2.
- Added consistency policy controls with snapshot settings.
- Added explicit WebDAV raw-tree direct-upload mode with bounded request/upload limits.
- Added alert digest support and de-noised run badges.
- Added dashboard overview API and redesigned dashboard overview UX.
- Added Job Detail page and dedicated Job Detail actions toolbar.
- Added UI theme presets and Appearance theme picker.
- Added background style options (`aurora`, `solid`, `plain`).
- Added in-app docs serving under `/docs`.
- Added VitePress docs site structure and EN/ZH i18n docs support.
- Added UI/docs locale synchronization.
- Added CLI help i18n and generated CLI reference docs.
- Added `doctor`/`config` commands and readiness/ops improvements.
- Added Linux deb/rpm installers, Linux musl artifacts, macOS artifacts, and Windows MSI packaging support.
- Added systemd and Windows Service support.
- Added a single-file `CHANGELOG.md` workflow as the release-notes source.
- Added `scripts/release-preflight.sh` for release preflight validation/extraction.

### Changed
- Unified Settings navigation to a single source of truth.
- Refactored Jobs into a workspace-style layout with better context and scanning.
- Added Jobs workspace layout/view modes and improved toolbar behavior.
- Improved Jobs overview metadata cards and section organization.
- Standardized list/toolbar patterns across Agents/Jobs/Snapshots/Settings screens.
- Improved dashboard first-paint behavior and deferred heavy sections.
- Improved refresh cancellation and refresh ordering stability.
- Moved Agents and Jobs listing to server-side pagination/filtering for scale.
- Improved node-context navigation and preferred-node cues.
- Reworked Run Detail information hierarchy, density, and desktop layout.
- Standardized Web UI design semantics with token-driven styling.
- Improved dark-mode consistency and contrast behavior across themes.
- Switched archive uploads to rolling part upload to reduce local disk peak usage.
- Standardized structured API error contracts across backend and UI.
- Switched GitHub release notes generation to curated `CHANGELOG.md` sections.
- Added project workflow rule requiring changelog updates for user-visible changes.

### Fixed
- Fixed missing About entry wiring in Settings sidebar.
- Fixed desktop modal height jumps in Job Editor and picker dialogs.
- Fixed picker table height fill behavior and double-scroll issues.
- Fixed picker current-directory confirmation/footer interaction flow.
- Fixed breadcrumb truncation/overflow behavior in picker path bars.
- Fixed WebDAV restore MKCOL 409 behavior by creating parent collections.
- Fixed final progress flush behavior to avoid 99% stuck states.
- Fixed duplicate overall progress display and grid regressions in Run Detail.
- Fixed run events filter/action wrapping and width collisions across breakpoints.
- Fixed jobs split-pane toolbar overlap and list/filter alignment issues.
- Fixed bulk selection validity across paged Jobs lists.
- Fixed wheel-scroll blocking in Job Detail tabs.
- Fixed dark-mode background flash and theme override application gaps.
- Fixed neutral-surface behavior for plain background style in dark mode.
- Fixed white-screen crash caused by CSS variable override paths.
- Fixed snapshot index upsert idempotency.
- Fixed snapshot delete-event append concurrency behavior.
- Fixed archive cascade snapshot delete pagination gaps.
- Fixed retention preview/apply override payload validation.
- Fixed retention apply reporting when delete tasks already exist.
- Fixed Windows file-id instability effects in consistency fingerprints.
- Fixed setup weak-password minimum-length error display.
- Fixed multiple CI/cross-platform release-blocking issues on Windows/macOS/Linux.

### Security
- Hardened Agent streams and edge trust boundaries.
- Hardened WebDAV direct-upload safety boundaries with explicit limits.
- Remediated CodeQL secret-hygiene alerts.
- Remediated dependency security alerts across Rust/UI/Docs stacks.
- Strengthened gitleaks scanning workflow and false-positive handling in CI.

## [v0.1.0] - 2026-01-13

### Added
- Initial open-source release of Bastion (Hub + optional Agent + Web UI), including downloadable release artifacts.
