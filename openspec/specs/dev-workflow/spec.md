# dev-workflow Specification

## Purpose
TBD - created by archiving change add-windows-tray-autostart-subcommand. Update Purpose after archive.
## Requirements
### Requirement: Windows Installer Must Provide Tray Startup Entry
The Windows MSI package SHALL install a startup entry that launches Bastion tray mode on user logon.

#### Scenario: user logs in after MSI install
- **WHEN** Bastion has been installed via MSI and the user signs into Windows
- **THEN** Windows Startup launches `bastion tray run` automatically
- **AND** a Bastion tray icon is available without manually running commands

### Requirement: Windows Tray Mode Must Be Exposed as a Subcommand
Bastion SHALL expose a Windows-only tray runtime as a CLI subcommand so installer/startup entries can reuse the same executable.

#### Scenario: tray mode is launched via CLI
- **WHEN** `bastion tray run` is executed on Windows
- **THEN** Bastion runs as a tray process with menu actions for opening Web UI and controlling the Bastion service
- **AND** tray mode does not start the normal Hub foreground server flow

### Requirement: MSI Install Must Enable Machine Boot Startup For Hub Service
The Windows MSI service installation SHALL configure the Bastion service for automatic startup at system boot.

#### Scenario: machine reboots after installation
- **WHEN** Windows boots after Bastion MSI install
- **THEN** Service Control Manager attempts to start the `Bastion` service automatically
- **AND** users do not need to manually start the service from Services UI for normal startup

### Requirement: Windows Install Completion Launch Must Not Block Finish UI
The Windows MSI completion option to open Bastion Web UI SHALL avoid long-running blocking behavior in the installer finish action.

#### Scenario: user keeps launch checkbox selected and clicks Finish
- **WHEN** install completes and the launch option remains selected
- **THEN** the finish action returns promptly without waiting for long readiness loops inside MSI UI thread
- **AND** Bastion Web UI is opened through the user's default browser shell after install finishes

### Requirement: Windows Install Execution Must Start Bastion Service for First Launch
The Windows MSI install execution sequence SHALL attempt to start the Bastion service so first-run launch can succeed without manual service startup.

#### Scenario: first install completes
- **WHEN** installation finalizes
- **THEN** installer service-control actions attempt to start the `Bastion` service
- **AND** launch workflows rely on that service state instead of service-start logic inside finish-button UI actions

### Requirement: Windows Data-Removal Uninstall Uses Explicit Entry Point
Windows uninstall invoked from standard OS app-management paths SHALL keep Bastion data by default, while providing an explicit packaged entry point for uninstall-and-delete-data behavior.

#### Scenario: user uninstalls from Windows Settings Apps list
- **WHEN** uninstall is invoked via standard Apps/Installed apps entry
- **THEN** uninstall keeps `C:\ProgramData\bastion` data unless an explicit MSI property override is supplied

#### Scenario: user selects explicit uninstall-remove-data shortcut
- **WHEN** user triggers the packaged "uninstall and remove data" entry point
- **THEN** uninstall is launched with MSI property override enabling `C:\ProgramData\bastion` cleanup

### Requirement: Windows Installer Completion Flow Must Support Guided Launch
The Windows MSI interactive completion flow SHALL provide a default-checked option to launch Bastion for first-time installs.

#### Scenario: user keeps default completion option selected
- **WHEN** installation finishes and the user clicks Finish with the launch option selected
- **THEN** the installer attempts to start the Bastion Windows service first
- **AND** only opens the Bastion Web UI after local service readiness is observed

#### Scenario: user unchecks completion option
- **WHEN** installation finishes and the launch option is not selected
- **THEN** the installer does not auto-open the Bastion Web UI

### Requirement: Windows Uninstall Prompt Must Appear Across Interactive Remove Entry Points
The Windows MSI interactive uninstall flow SHALL present the Bastion data-retention choice before uninstall confirmation across supported interactive remove entry paths.

#### Scenario: user enters uninstall from direct remove path
- **WHEN** interactive uninstall reaches the remove confirmation stage
- **THEN** the installer first shows the Bastion data-retention dialog
- **AND** preserves the default behavior of keeping data unless deletion is explicitly selected

### Requirement: Windows MSI x86_64 Packages Must Install as 64-bit
The Windows MSI packaging flow SHALL build x86_64 installers as 64-bit packages so Bastion installs to the 64-bit Program Files location.

#### Scenario: x86_64 release MSI is installed on a 64-bit host
- **WHEN** a user installs the `x86_64-pc-windows-msvc` MSI package
- **THEN** application files are installed under `Program Files` (64-bit)
- **AND** the MSI component architecture metadata is marked as 64-bit

### Requirement: Windows MSI Must Provide Guided UX and Start Menu Entries
The Windows MSI SHALL include a standard guided installer UI and create Start Menu shortcuts for primary Bastion entry points.

#### Scenario: user runs the MSI interactively
- **WHEN** the installer starts in normal UI mode
- **THEN** the installer provides guided setup dialogs instead of progress-only flow
- **AND** installation creates Start Menu entries for launching Bastion and opening the local Bastion Web UI

### Requirement: Windows Uninstall Must Offer Optional Data Cleanup
The Windows MSI uninstall flow SHALL present an explicit option to remove Bastion data under `C:\ProgramData\bastion`, defaulting to keep data.

#### Scenario: user uninstalls with default options
- **WHEN** uninstall proceeds without selecting data removal
- **THEN** `C:\ProgramData\bastion` remains on disk

#### Scenario: user selects data removal during uninstall
- **WHEN** uninstall proceeds with data removal selected
- **THEN** installer cleanup removes `C:\ProgramData\bastion` recursively

### Requirement: Windows MSI Metadata Must Be User-Friendly
The Windows MSI SHALL expose meaningful Add/Remove Programs metadata including non-empty product version and support links.

#### Scenario: user checks installed Bastion app details
- **WHEN** Bastion appears in Windows Apps & features
- **THEN** displayed metadata includes a non-placeholder version value derived from release package metadata
- **AND** publisher/support information is populated from installer metadata

### Requirement: Manual Release Builds Use Tag-Based Preview Version Labels
The release workflow SHALL derive manual `workflow_dispatch` build version labels from the latest repository release tag and the current commit short hash.

#### Scenario: workflow_dispatch computes preview build version
- **WHEN** the release workflow runs via `workflow_dispatch`
- **THEN** it resolves the latest tag matching `v*`
- **AND** computes a preview label in the format `<tag-without-v>-dh<short-hash>`
- **AND** uses that label for build metadata and packaged asset naming in manual build artifacts

### Requirement: Tag-Triggered Release Versioning Remains Stable
Tag-triggered release workflow behavior SHALL remain unchanged for release labels and package version derivation.

#### Scenario: tag push keeps existing release label behavior
- **WHEN** a tag matching `v*` triggers the release workflow
- **THEN** release labels keep using the pushed tag name
- **AND** package semantic version values are derived from that same tag

### Requirement: Windows MSI Release Asset Must Be Self-Contained
The release workflow SHALL produce a Windows MSI asset that includes the Bastion executable payload.

#### Scenario: Windows MSI package is built in release workflow
- **WHEN** the Windows packaging job creates the MSI artifact
- **THEN** the MSI contains embedded payload data required for installation
- **AND** the workflow fails if the MSI output is clearly invalid (for example, unexpectedly tiny size)

### Requirement: Manual Release Build Artifacts Use Per-File Granularity
Manual `workflow_dispatch` release builds SHALL upload artifacts with per-file granularity equivalent to published release assets.

#### Scenario: Manual release workflow packages Linux GNU outputs
- **WHEN** the Linux GNU build produces `.tar.gz`, `.deb`, and `.rpm` outputs
- **THEN** each output is uploaded as a separate artifact
- **AND** artifacts are named to match the underlying packaged file

### Requirement: Release Preflight Script Validates Changelog Readiness
The project SHALL provide a release preflight script that validates changelog readiness for a target release tag.

#### Scenario: Maintainer runs preflight for a release tag
- **WHEN** a maintainer runs the preflight script with `--tag vX.Y.Z`
- **THEN** the script runs changelog structure checks
- **AND** the script runs changelog regression tests
- **AND** the script extracts release notes for the target tag into an output file

#### Scenario: Invalid tag or missing changelog section blocks preflight
- **WHEN** the preflight script receives an invalid tag format or a tag without matching changelog section
- **THEN** the script exits with a non-zero status
- **AND** prints a clear error message describing the failure

### Requirement: Release Workflow Uses Preflight Script
The GitHub release workflow SHALL use the release preflight script for changelog validation and release-note generation.

#### Scenario: Tag push runs preflight before publishing release
- **WHEN** a `v*` tag triggers the release workflow
- **THEN** the workflow runs the release preflight script with the workflow tag
- **AND** uses the preflight-generated notes file as the GitHub Release body

### Requirement: Repository Maintains A Single Changelog File
The repository SHALL keep a root-level `CHANGELOG.md` as the authoritative changelog for user-facing product changes.

#### Scenario: User-visible changes are recorded in Unreleased
- **WHEN** a change affects end-user behavior, UX, compatibility, or operational guidance
- **THEN** contributors add an entry under `## [Unreleased]` in `CHANGELOG.md`
- **AND** entries are grouped into standard categories (`Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`)
- **AND** internal-only maintenance work (for example CI/spec chores without user impact) MAY be omitted

### Requirement: Release Notes Come From Versioned Changelog Sections
The release workflow SHALL generate GitHub Release notes from the matching version section in `CHANGELOG.md`.

#### Scenario: Tag push publishes matching changelog section
- **WHEN** a tag matching `v*` is pushed
- **THEN** the workflow extracts the section `## [<tag>]` (or equivalent semver heading) from `CHANGELOG.md`
- **AND** publishes that extracted content as the GitHub Release body

#### Scenario: Missing changelog section blocks release publication
- **WHEN** a release tag does not have a matching version section in `CHANGELOG.md`
- **THEN** release notes generation fails
- **AND** the release publish job fails before creating/updating the GitHub Release

### Requirement: CI Validates Changelog Tooling
Project CI SHALL validate changelog format and extraction behavior.

#### Scenario: Changelog structure is invalid
- **WHEN** changelog validation runs in CI
- **THEN** CI fails if `CHANGELOG.md` is missing required top-level sections or uses unsupported category headings

#### Scenario: Extraction regression is introduced
- **WHEN** changelog extraction regression tests run in CI
- **THEN** CI fails if extraction no longer returns the expected version section output

### Requirement: UI Tests Reuse Shared Naive-UI Stub Helpers
UI unit tests SHALL reuse shared Naive UI stub helpers for common components to reduce duplication and avoid repeated warning-prone stub behavior.

#### Scenario: Multiple view specs import shared stubs
- **GIVEN** view specs need Naive UI component stubs
- **WHEN** tests are implemented or updated
- **THEN** specs import common stub helpers instead of redefining identical base stubs
- **AND** shared input stubs safely avoid invalid native prop forwarding warnings

### Requirement: Router Meta Configuration Uses Shared Builders for Repeated Blocks
When route meta structures are repeated across multiple routes, router config SHALL use shared helper builders to keep metadata consistent and reduce drift.

#### Scenario: Repeated settings meta uses a shared builder
- **GIVEN** multiple settings routes share title/back-navigation meta patterns
- **WHEN** router meta is defined
- **THEN** shared helper builders generate repeated meta blocks
- **AND** resulting route behavior remains unchanged

### Requirement: UI Unit Tests Avoid Stub-Generated Native Prop Warnings
When UI unit tests stub third-party components with native elements, test stubs SHALL avoid forwarding invalid native-only props that produce avoidable Vue runtime warnings.

#### Scenario: Stubbed input does not emit invalid native size warning
- **GIVEN** a UI unit test stubs an input-like component with a native `<input>` element
- **WHEN** the test renders a view that passes third-party component props (for example, `size=\"small\"`)
- **THEN** the stub handles or filters that prop safely
- **AND** the test run does not emit an avoidable warning for invalid native input size assignment

### Requirement: CI Enforces UI Static Quality Checks
The project CI workflow SHALL run frontend static quality checks for the `ui/` workspace, including:
- ESLint in non-mutating check mode
- Vue/TypeScript type-check

These checks SHALL fail CI when violations are present.

#### Scenario: UI lint violation fails CI
- **GIVEN** a UI source file contains an ESLint violation
- **WHEN** `scripts/ci.sh` is executed in CI
- **THEN** the UI lint check step fails
- **AND** later build/test steps do not report overall success

#### Scenario: UI type-check violation fails CI
- **GIVEN** a UI source file contains a TypeScript type error
- **WHEN** `scripts/ci.sh` is executed in CI
- **THEN** the UI type-check step fails
- **AND** later build/test steps do not report overall success

### Requirement: Workspace Centralizes Shared Dependency Versions
The Rust workspace SHALL centralize versions for dependencies used across multiple crates by declaring them in `[workspace.dependencies]`.

#### Scenario: Shared dependency versions are defined in one place
- **WHEN** a shared dependency version is updated
- **THEN** it is updated in the root `Cargo.toml` under `[workspace.dependencies]`
- **AND** all workspace crates consume that version via `workspace = true`

### Requirement: Member Crates Use `workspace = true` For Centralized Dependencies
Workspace member crates MUST reference centralized dependencies using `workspace = true` to avoid per-crate version drift.

#### Scenario: Cargo build uses a consistent version set
- **WHEN** the workspace is built (`cargo test --workspace`)
- **THEN** Cargo resolves consistent dependency versions across all workspace members without per-crate version pinning for centralized dependencies

### Requirement: Workspace Avoids `tokio/full`
The Rust workspace MUST NOT enable `tokio`'s `full` feature and MUST instead explicitly declare only the Tokio feature flags required by the codebase.

#### Scenario: CI rejects `tokio/full` in any workspace crate
- **GIVEN** a workspace member declares `tokio` with `features` containing `"full"`
- **WHEN** CI runs the repository checks
- **THEN** the CI job fails with guidance to keep Tokio features minimal

#### Scenario: Workspace builds with minimal Tokio feature flags
- **WHEN** the backend workspace is built and tested (`cargo test --workspace`)
- **THEN** the build succeeds without relying on `tokio/full`

### Requirement: Linux Packages Install A systemd Unit (No Auto-Start)
The project SHALL ship a `systemd` unit file with Linux `.deb` and `.rpm` release packages.

#### Scenario: Installing a Linux package provides a startable systemd service
- **WHEN** a user installs the `.deb` or `.rpm`
- **THEN** `bastion.service` is installed
- **AND** the service is **NOT** started automatically
- **AND** documentation explains how to reload systemd and start/enable the service

### Requirement: Windows MSI Installs A Windows Service (No Auto-Start)
The project SHALL install a Windows Service entry as part of the MSI installer.

#### Scenario: Installing the MSI provides a startable Windows Service
- **WHEN** a user installs the MSI
- **THEN** the Bastion service is installed
- **AND** the service is **NOT** started automatically
- **AND** documentation explains how to start the service after installation

### Requirement: Service Stop Performs A Graceful Shutdown
The Hub SHALL support graceful shutdown when managed by service managers.

#### Scenario: systemd stop triggers graceful shutdown
- **WHEN** systemd stops the service (SIGTERM)
- **THEN** the Hub stops accepting new requests and shuts down gracefully

#### Scenario: Windows Service stop triggers graceful shutdown
- **WHEN** the Windows Service receives a stop/shutdown control signal
- **THEN** the Hub stops accepting new requests and shuts down gracefully

### Requirement: GitHub Releases Publish Installers And macOS Binaries
The project SHALL publish platform-appropriate release artifacts on GitHub Releases for each `v*` tag.

#### Scenario: Tag push creates a GitHub Release with installers and checksums
- **WHEN** a tag matching `v*` is pushed
- **THEN** GitHub Actions builds `bastion` with embedded UI+docs for:
  - Linux x64
  - Windows x64
  - macOS x64
  - macOS arm64
- **AND** the workflow publishes a GitHub Release containing:
  - Linux: `tar.gz`, `.deb`, `.rpm`
  - Windows: `.zip`, `.msi` (MSI MUST NOT add Bastion to `PATH` by default)
  - macOS: archives for x64 and arm64
  - `sha256sums.txt` covering all uploaded artifacts

### Requirement: CLI Reference Docs Are Auto-Generated
The repository SHALL provide an automated way to generate a CLI reference page from the actual CLI definition.

The generated reference MUST include English and Chinese variants.

#### Scenario: Doc generator produces CLI reference pages
- **WHEN** the doc generator is run in write mode
- **THEN** it writes an English CLI reference page under `docs/user/reference/`
- **AND** it writes a Chinese CLI reference page under `docs/zh/user/reference/`

### Requirement: CI Fails On Missing CLI Translations
CI MUST fail when the CLI help translation resource is missing any required `zh-CN` key derived from the CLI definition.

#### Scenario: Missing translation key fails CI
- **GIVEN** a required translation key is missing from the `zh-CN` translation map
- **WHEN** CI runs the repository checks
- **THEN** the CI job fails

### Requirement: CI Fails On Stale Generated Reference Docs
CI MUST fail when the generated CLI reference pages differ from the committed outputs.

#### Scenario: Generated output drift fails CI
- **GIVEN** the CLI definition changed
- **WHEN** CI runs the doc generator in check mode
- **THEN** the CI job fails until the generated docs are updated and committed

### Requirement: CI Can Build Docs Site For Embedded Builds
The repository CI workflow SHALL be able to build the docs site so that `embed-docs` builds succeed.

#### Scenario: CI builds docs with /docs base
- **WHEN** CI runs the repository checks
- **THEN** it builds the docs site with `DOCS_BASE=/docs/`

### Requirement: Documentation Site Is Published To GitHub Pages
The project SHALL publish a documentation site built from the repository `docs/` content to GitHub Pages at the entry path `/<repo>/docs/`.

#### Scenario: GitHub Pages deploys docs site under /docs/
- **WHEN** a commit is pushed to `main`
- **THEN** GitHub Actions builds the docs site
- **AND** deploys it to GitHub Pages with the entry path `/<repo>/docs/`

### Requirement: GitHub Release Includes Changelog Body
Each GitHub Release SHALL include a human-readable changelog section in the release body.

#### Scenario: Tag push generates release notes from git history
- **WHEN** a tag matching `v*` is pushed
- **THEN** the release workflow generates release notes from git commits since the previous tag (or from repository start for the first tag)
- **AND** the release body contains a changelog section and a link to the full compare view

### Requirement: GitHub Releases Publish Prebuilt Binaries
The project SHALL provide a GitHub Actions release workflow that publishes prebuilt binaries for Linux x64 and Windows x64.

#### Scenario: Tag push creates a GitHub Release with binaries
- **WHEN** a tag matching `v*` is pushed
- **THEN** GitHub Actions builds `bastion` for Linux x64 and Windows x64
- **AND** the workflow publishes a GitHub Release containing the two artifacts

### Requirement: Repository Is Published With License And CI
The project SHALL be publishable as an open-source GitHub repository with a clear OSI-approved license and a baseline CI workflow that runs the project checks.

#### Scenario: GitHub Actions runs repo CI script
- **WHEN** a commit is pushed or a pull request is opened
- **THEN** GitHub Actions runs `bash scripts/ci.sh`
- **AND** the job fails on formatting, lint, tests, or secret scanning failures

### Requirement: CI Includes Automated Secret Leak Scanning
The project CI workflow SHALL run an automated secret leak scan to detect likely committed credentials (tokens, API keys, private keys) before changes are merged or released.

#### Scenario: CI fails when a likely secret is detected
- **GIVEN** the repository contains content that matches a secret leak rule
- **WHEN** the CI scripts are executed
- **THEN** the secret scan step fails the run
- **AND** the output is redacted to avoid printing secrets in plaintext logs

### Requirement: Config Reference Docs Are Auto-Generated
The repository SHALL provide an automated way to generate configuration/environment reference documentation derived from the actual CLI/env definitions.

The generated reference MUST include English and Chinese variants.

#### Scenario: Doc generator produces config reference pages
- **WHEN** the doc generator is run in write mode
- **THEN** it writes an English config reference page under `docs/user/reference/`
- **AND** it writes a Chinese config reference page under `docs/zh/user/reference/`

### Requirement: CI Fails On Stale Generated Config Reference Docs
CI MUST fail when the generated config reference pages differ from the committed outputs.

#### Scenario: Generated output drift fails CI
- **GIVEN** the CLI/env definition changed
- **WHEN** CI runs the doc generator in check mode
- **THEN** the CI job fails until the generated docs are updated and committed

