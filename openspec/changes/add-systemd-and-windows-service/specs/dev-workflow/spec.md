## ADDED Requirements

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
