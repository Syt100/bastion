use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use single_instance::SingleInstance;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use windows_service::service::{ServiceAccess, ServiceState};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use windows_sys::Win32::System::Console::GetConsoleWindow;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostQuitMessage, SW_HIDE, SW_SHOWNORMAL, ShowWindow,
    TranslateMessage,
};

use crate::config::{TrayArgs, TrayCommand};

const SERVICE_NAME: &str = "Bastion";
const TRAY_INSTANCE_ID: &str = "global.bastion.tray";
const WEB_UI_URL: &str = "http://127.0.0.1:9876/";

const SERVICE_WAIT_TIMEOUT: Duration = Duration::from_secs(25);
const WEB_UI_WAIT_TIMEOUT: Duration = Duration::from_secs(25);
const WEB_UI_PROBE_TIMEOUT: Duration = Duration::from_secs(2);
const POLL_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) fn run(args: TrayArgs) -> Result<(), anyhow::Error> {
    match args.command {
        TrayCommand::Run => run_tray(),
    }
}

fn run_tray() -> Result<(), anyhow::Error> {
    // Prevent duplicate tray processes from stacking icons when startup/manual launches overlap.
    let _instance =
        SingleInstance::new(TRAY_INSTANCE_ID).context("failed to create tray single-instance")?;
    if !_instance.is_single() {
        return Ok(());
    }

    hide_console_window();

    let open_web_ui = MenuItem::with_id("open-web-ui", "Open Bastion Web UI", true, None);
    let start_service = MenuItem::with_id("start-service", "Start Bastion Service", true, None);
    let stop_service = MenuItem::with_id("stop-service", "Stop Bastion Service", true, None);
    let separator = PredefinedMenuItem::separator();
    let exit_tray = MenuItem::with_id("exit-tray", "Exit Tray", true, None);

    let tray_menu = Menu::new();
    tray_menu
        .append_items(&[
            &open_web_ui,
            &start_service,
            &stop_service,
            &separator,
            &exit_tray,
        ])
        .context("failed to build tray menu")?;

    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("Bastion")
        .with_icon(load_tray_icon()?)
        .with_menu(Box::new(tray_menu))
        .build()
        .context("failed to create tray icon")?;

    let open_web_ui_id = open_web_ui.id().clone();
    let start_service_id = start_service.id().clone();
    let stop_service_id = stop_service.id().clone();
    let exit_tray_id = exit_tray.id().clone();

    loop {
        let mut msg = MSG::default();
        let code = unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) };
        if code == -1 {
            anyhow::bail!("Windows message loop failed");
        }
        if code == 0 {
            break;
        }

        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if handle_menu_event(
                event,
                &open_web_ui_id,
                &start_service_id,
                &stop_service_id,
                &exit_tray_id,
            )? {
                unsafe {
                    PostQuitMessage(0);
                }
            }
        }
    }

    Ok(())
}

fn handle_menu_event(
    event: MenuEvent,
    open_web_ui_id: &MenuId,
    start_service_id: &MenuId,
    stop_service_id: &MenuId,
    exit_tray_id: &MenuId,
) -> Result<bool, anyhow::Error> {
    if event.id == *open_web_ui_id {
        thread::spawn(|| {
            if let Err(error) = open_web_ui_after_readiness() {
                eprintln!("failed to open Bastion Web UI from tray: {error:?}");
            }
        });
        return Ok(false);
    }

    if event.id == *start_service_id {
        thread::spawn(|| {
            if let Err(error) = ensure_service_running(SERVICE_WAIT_TIMEOUT) {
                if is_access_denied(&error) {
                    if let Err(elevate_error) = run_service_command_elevated("start") {
                        eprintln!(
                            "failed to start Bastion service from tray (and failed to request admin approval): {elevate_error:?}"
                        );
                    }
                } else {
                    eprintln!("failed to start Bastion service from tray: {error:?}");
                }
            }
        });
        return Ok(false);
    }

    if event.id == *stop_service_id {
        thread::spawn(|| {
            if let Err(error) = stop_service(SERVICE_WAIT_TIMEOUT) {
                if is_access_denied(&error) {
                    if let Err(elevate_error) = run_service_command_elevated("stop") {
                        eprintln!(
                            "failed to stop Bastion service from tray (and failed to request admin approval): {elevate_error:?}"
                        );
                    }
                } else {
                    eprintln!("failed to stop Bastion service from tray: {error:?}");
                }
            }
        });
        return Ok(false);
    }

    if event.id == *exit_tray_id {
        return Ok(true);
    }

    Ok(false)
}

fn open_web_ui_after_readiness() -> Result<(), anyhow::Error> {
    // Opening the browser must work for non-admin users too.
    // We probe service readiness and only attempt service start as a best effort.
    if wait_for_web_ui(WEB_UI_PROBE_TIMEOUT).is_err() {
        match ensure_service_running(SERVICE_WAIT_TIMEOUT) {
            Ok(()) => {
                let _ = wait_for_web_ui(WEB_UI_WAIT_TIMEOUT);
            }
            Err(error) if is_access_denied(&error) => {}
            Err(error) => return Err(error),
        }
    }
    open_web_ui()
}

fn ensure_service_running(timeout: Duration) -> Result<(), anyhow::Error> {
    let service = open_service(ServiceAccess::QUERY_STATUS | ServiceAccess::START)?;
    let deadline = Instant::now() + timeout;
    let mut start_requested = false;

    loop {
        let state = service
            .query_status()
            .context("failed to query Bastion service status")?
            .current_state;

        match state {
            ServiceState::Running => return Ok(()),
            ServiceState::Stopped if !start_requested => {
                service
                    .start::<&str>(&[])
                    .context("failed to start Bastion service")?;
                start_requested = true;
            }
            ServiceState::Stopped => {}
            ServiceState::StartPending | ServiceState::StopPending => {}
            other => anyhow::bail!("Bastion service is in unsupported state: {other:?}"),
        }

        if Instant::now() >= deadline {
            anyhow::bail!("timed out waiting for Bastion service to reach running state");
        }

        thread::sleep(POLL_INTERVAL);
    }
}

fn stop_service(timeout: Duration) -> Result<(), anyhow::Error> {
    let service = open_service(ServiceAccess::QUERY_STATUS | ServiceAccess::STOP)?;

    if service
        .query_status()
        .context("failed to query Bastion service status")?
        .current_state
        == ServiceState::Stopped
    {
        return Ok(());
    }

    service
        .stop()
        .context("failed to request Bastion service stop")?;

    let deadline = Instant::now() + timeout;
    loop {
        let state = service
            .query_status()
            .context("failed to query Bastion service status")?
            .current_state;

        if state == ServiceState::Stopped {
            return Ok(());
        }

        if Instant::now() >= deadline {
            anyhow::bail!("timed out waiting for Bastion service to stop");
        }

        thread::sleep(POLL_INTERVAL);
    }
}

fn wait_for_web_ui(timeout: Duration) -> Result<(), anyhow::Error> {
    let deadline = Instant::now() + timeout;
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9876);

    loop {
        if TcpStream::connect_timeout(&addr.into(), Duration::from_millis(300)).is_ok() {
            return Ok(());
        }

        if Instant::now() >= deadline {
            anyhow::bail!("timed out waiting for Bastion Web UI to become reachable");
        }

        thread::sleep(POLL_INTERVAL);
    }
}

fn open_web_ui() -> Result<(), anyhow::Error> {
    Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(WEB_UI_URL)
        .spawn()
        .context("failed to launch browser for Bastion Web UI")?;
    Ok(())
}

fn open_service(access: ServiceAccess) -> Result<windows_service::service::Service, anyhow::Error> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .context("failed to open Windows service manager")?;
    manager
        .open_service(SERVICE_NAME, access)
        .context("failed to open Bastion Windows service")
}

fn load_tray_icon() -> Result<Icon, anyhow::Error> {
    // winres embeds the app icon as resource ordinal 1.
    if let Ok(icon) = Icon::from_resource(1, Some((32, 32))) {
        return Ok(icon);
    }

    let exe_path = std::env::current_exe().context("failed to resolve bastion executable path")?;
    let sidecar_icon = exe_path.with_file_name("bastion.ico");
    if let Ok(icon) = Icon::from_path(&sidecar_icon, Some((32, 32))) {
        return Ok(icon);
    }

    let mut rgba = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        rgba.extend_from_slice(&[45, 125, 255, 255]);
    }
    Icon::from_rgba(rgba, 16, 16).context("failed to create fallback tray icon")
}

fn run_service_command_elevated(action: &str) -> Result<(), anyhow::Error> {
    let verb = wide_null("runas");
    let file = wide_null("sc.exe");
    let parameters = wide_null(&format!("{action} {SERVICE_NAME}"));

    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            parameters.as_ptr(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    let status = result as isize;
    if status <= 32 {
        anyhow::bail!(
            "ShellExecuteW failed while requesting admin approval for `sc.exe {action} {SERVICE_NAME}` (status={status})"
        );
    }

    Ok(())
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn is_access_denied(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .and_then(|io_error| io_error.raw_os_error())
            == Some(5)
    })
}

fn hide_console_window() {
    unsafe {
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}
