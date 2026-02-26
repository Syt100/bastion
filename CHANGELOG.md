# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project follows [Semantic Versioning](https://semver.org/) while in pre-1.0 development.

## [Unreleased]

### Added
- Added optional notifications queue cursor pagination (`cursor` request + `next_cursor` response) to keep queued-results browsing stable under concurrent state changes.
- Added cancel actions for queued/running runs and running restore/verify operations in the Web UI.
- Added WebDAV upload tuning controls (`request_timeout_secs`, `connect_timeout_secs`, `max_put_attempts`) in job spec/API/UI to better match unstable or high-latency networks.

### Changed
- Changed snapshot listing API and Web UI pagination to use opaque keyset cursors (`next_cursor`) so pagination stays stable during concurrent snapshot status changes.
- Changed run/operation lifecycle handling to support graceful cancellation (`canceling` → `canceled`) with idempotent cancel requests and race-safe terminalization.
- Changed failed run events and Run Events UI to expose structured transport diagnostics (error code/kind/chain, retry/part/HTTP context, and operator hints).
- Changed run/maintenance/notification failure events to emit a unified `error_envelope` contract (schema version, stable code/kind, retriable metadata, i18n keys, transport protocol details, and context payloads) while keeping legacy fields for compatibility.
- Changed Run Events UI to render envelope-first localized diagnostics with graceful fallback, protocol-specific detail rows (HTTP/SFTP/provider fields), and async-operation/partial-failure panels.
- Changed Web UI i18n startup to lazy-load only the active locale before mount and load other locales on demand, reducing initial bundle payload.
- Changed Agent/Hub websocket relay paths to bounded queues with explicit backpressure handling to avoid unbounded memory growth under slow consumers.
- Changed CI checks to run `clippy` for both default-feature and all-features builds.
- Changed offline scheduler/writer internals to bounded queues with explicit full/closed handling for more predictable memory behavior under prolonged disconnects.
- Changed Dashboard desktop recent-runs idle prefetch to skip non-desktop viewports.
- Changed CI workflow action versions to current major lines (`actions/upload-pages-artifact` v4, `actions/upload-artifact` v6, `actions/cache` v5, `actions/checkout` v6, `actions/download-artifact` v7).
- Changed Web UI dependency baseline via grouped non-major upgrades (including Vue 3.5.29, Vite 7.3.1, Tailwind CSS 4.2.1, and Vitest 4.0.18).
- Changed Rust dependency baseline via grouped non-major upgrades (including Tokio 1.49, UUID 1.21, Chrono 0.4.44, Clap 4.5.60, and Tempfile 3.26).
- Changed XML/runtime dependency baselines by upgrading `roxmltree` to 0.21.1 and `windows-service` to 0.8.0.

### Deprecated
- _No user-facing changes yet._

### Removed
- _No user-facing changes yet._

### Fixed
- Fixed Agent heartbeat persistence overhead by throttling `agents.last_seen_at` updates to reduce DB write amplification during high message throughput.
- Fixed docs HTTP test locking to avoid holding sync mutex guards across `await` boundaries.
- Fixed notifications queue pagination continuity when earlier rows leave the filtered set between page fetches.
- Fixed Web UI rapid locale toggles to enforce last-write-wins behavior and avoid stale locale activation from slower async loads.
- Fixed rolling archive upload failures to preserve the underlying uploader/network root cause instead of surfacing a generic `rolling uploader dropped`.
- Fixed WebDAV upload retry handling to classify failure kinds (auth, timeout, rate-limit, payload-too-large, transport) and retry only when appropriate.
- Fixed fallback failure classification to mark connect/DNS/routing transport errors (for example `connection refused`) as `network` with actionable hints.
- Fixed WebDAV read/delete/download failure mapping to preserve HTTP diagnostics and avoid retrying non-retriable auth/config failures.
- Fixed cleanup/artifact-delete run events to include actionable `hint` fields, and localized the Run Events detail hint label for zh/en UI.

### Security
- Remediated the open `glib` dependency alert path (`GHSA-wrw7-89jp-8q8g`) by switching Windows tray integration to a Windows-only tray crate.
- Refreshed CI/UI/Rust dependency sets through Dependabot merges to reduce known-vulnerability exposure and keep patch-level security fixes current.

## [v0.2.2] - 2026-02-24

### Added
- Added Windows tray mode (`bastion tray run`) with tray actions to open Web UI and start/stop the Bastion service.
- Added a Windows startup entry that launches Bastion Tray automatically after user sign-in.
- Added initial Bastion brand icon assets (`assets/branding`) with a blue shield + sync ring style.
- Added Windows executable icon resource embedding so tray/shortcut icons use the branded icon.

### Changed
- Changed Windows MSI service install mode to auto-start `Bastion` on system boot.
- Changed Web UI favicon from the default Vue icon to the new Bastion brand icon.
- Changed Windows MSI metadata to register `ARPPRODUCTICON` and apply branded icons on Start Menu/Startup shortcuts.
- Changed release preflight checks to validate required branding icon assets before release extraction.
- Changed Windows tray Start Menu/Startup shortcuts to always write tray logs to `C:\ProgramData\bastion\logs\tray.log` with daily rotation.
- Changed docs to include Windows tray usage, tray log defaults, and `BASTION_TRAY_KEEP_CONSOLE=1` debug behavior.

### Fixed
- Fixed Windows tray icon loading to use embedded icon resources (with `.ico` sidecar fallback) instead of attempting to load icons from `bastion.exe` file paths.
- Fixed tray actions for standard users by avoiding privileged service access in "Open Web UI" flow and adding UAC-elevated fallback for Start/Stop service actions when access is denied.
- Fixed tray launch behavior by detaching from the console window by default (with `BASTION_TRAY_KEEP_CONSOLE=1` debug override).
- Fixed missing tray logs by initializing structured logging in `bastion tray run`.
- Fixed tray "Open Bastion Web UI" failures on some Windows hosts by using `ShellExecuteW` URL open behavior directly (instead of `rundll32.exe` invocation).

## [v0.2.1] - 2026-02-13

### Added
- Added a guided Windows MSI installer flow (license + install directory dialogs) instead of progress-only UX.
- Added Windows Start Menu shortcuts for `Bastion`, `Bastion Web UI`, and `Uninstall Bastion (Remove data)`.
- Added an install-completion checkbox (default enabled) to start Bastion and open Web UI after installation.

### Changed
- Changed release `workflow_dispatch` artifact uploads to per-file granularity so Linux GNU `.tar.gz`, `.deb`, and `.rpm` outputs are downloaded separately (matching published release assets).
- Changed release `workflow_dispatch` version labeling to `<latest-tag>-dh<short-hash>` for build metadata and artifact naming, while keeping tag-triggered release labels unchanged.
- Changed Windows x86_64 MSI packaging to explicit 64-bit mode so installs target `Program Files` instead of `Program Files (x86)` on 64-bit hosts.
- Changed Windows installer metadata to include support/about/update links in Apps & features.
- Changed default Windows uninstall behavior to keep `C:\ProgramData\bastion` data, with explicit data-removal uninstall entry in Start Menu.

### Fixed
- Fixed Windows MSI packaging to embed installer CAB payloads so release MSI artifacts contain the expected application payload.
- Added release workflow sanity checks to fail Windows packaging if the generated MSI is unexpectedly tiny.
- Fixed install completion hangs by making Web UI launch action non-blocking and starting service during install execution.
- Fixed install completion launch behavior to avoid transient console flashes and use shell-based browser opening.
- Fixed explicit "Uninstall Bastion (Remove data)" path by correctly wiring deferred cleanup command data for `BASTION_REMOVE_DATA=1`.

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
