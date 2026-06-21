mod commands;
mod devices;
mod gpu;
mod icons;

use std::sync::{Arc, Mutex};

use commands::{get_devices, get_gpu_usage, get_process_icon, kill_process, AppState};
use gpu::GpuState;
use sysinfo::System;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Avvia il campionamento GPU su thread in background.
    let gpu_state = Arc::new(GpuState::default());
    gpu::start(gpu_state.clone());

    let app_state = AppState {
        gpu: gpu_state,
        sys: Mutex::new(System::new()),
        nvml: Mutex::new(devices::init_nvml()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(app_state)
        .setup(|app| {
            // --- System tray ---
            let show = MenuItem::with_id(app, "show", "Mostra GPUst", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Esci", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("GPUst — GPU Task Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Minimize-to-tray: chiudere la finestra la nasconde soltanto.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_gpu_usage,
            get_devices,
            get_process_icon,
            kill_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_main<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
