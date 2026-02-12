## ADDED Requirements

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
