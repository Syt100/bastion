## ADDED Requirements

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
