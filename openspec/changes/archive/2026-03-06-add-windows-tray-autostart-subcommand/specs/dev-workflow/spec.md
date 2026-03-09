## ADDED Requirements

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
