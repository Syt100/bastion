## ADDED Requirements

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
