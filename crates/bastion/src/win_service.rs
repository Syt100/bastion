use std::ffi::OsString;
use std::time::Duration;

use anyhow::Context as _;
use clap::{CommandFactory, FromArgMatches as _};
use tokio_util::sync::CancellationToken;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

use crate::config::{Cli, ServiceArgs, ServiceCommand};

const SERVICE_NAME: &str = "Bastion";

windows_service::define_windows_service!(ffi_service_main, service_main);

pub(crate) fn run(args: ServiceArgs) -> Result<(), anyhow::Error> {
    match args.command {
        ServiceCommand::Run => {
            windows_service::service_dispatcher::start(SERVICE_NAME, ffi_service_main)
                .context("failed to start Windows service dispatcher")?
        }
    }
    Ok(())
}

fn service_main(_arguments: Vec<OsString>) {
    if let Err(error) = service_main_inner() {
        // Best-effort reporting; service environments may not have a visible console.
        eprintln!("bastion service error: {error:?}");
    }
}

fn service_main_inner() -> Result<(), anyhow::Error> {
    let shutdown = CancellationToken::new();
    let shutdown_signal = shutdown.clone();

    let status_handle =
        service_control_handler::register(SERVICE_NAME, move |event| match event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                shutdown_signal.cancel();
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        })?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build Tokio runtime for Windows service")?;

    let result = rt.block_on(async move {
        // Service mode should be configured via environment variables (system-wide env).
        // We parse `bastion` with no CLI args so clap applies defaults and reads `BASTION_*` env vars.
        let locale = crate::i18n::cli::resolve_cli_locale();
        let cmd = crate::i18n::cli::localize_command(Cli::command(), locale);
        let matches = cmd.get_matches_from(vec![OsString::from("bastion")]);
        let Cli {
            command,
            hub,
            mut logging,
        } = Cli::from_arg_matches(&matches)?;

        if command.is_some() {
            anyhow::bail!("service runner received an unexpected subcommand");
        }

        // A Windows Service typically has no interactive console; default to a log file unless
        // the user explicitly configured `--log-file` or `BASTION_LOG_FILE`.
        if logging.log_file.is_none()
            && std::env::var_os("BASTION_LOG_FILE").is_none()
            && let Ok(program_data) = std::env::var("PROGRAMDATA")
        {
            logging.log_file = Some(
                std::path::PathBuf::from(program_data)
                    .join("bastion")
                    .join("logs")
                    .join("bastion.log"),
            );
        }

        crate::run_hub(hub, logging, &matches, shutdown.clone()).await
    });

    let exit_code = match result {
        Ok(()) => ServiceExitCode::Win32(0),
        Err(error) => {
            eprintln!("bastion service hub error: {error:?}");
            ServiceExitCode::Win32(1)
        }
    };

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code,
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}
