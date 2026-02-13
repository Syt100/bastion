## ADDED Requirements

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
