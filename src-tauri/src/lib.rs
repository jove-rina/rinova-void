mod clash;
mod commands;
mod sidecar;
mod shortcut;
mod tray;
mod window;

use tauri::{Manager, RunEvent, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            clash::setup(app)?;
            if let Err(e) = window::init_main_window(app.handle()) {
                log::warn!("窗口初始化: {}", e);
            }
            if let Err(e) = tray::setup(app.handle()) {
                log::warn!("系统托盘: {}", e);
            }
            if let Err(e) = shortcut::setup(app.handle()) {
                log::warn!("全局快捷键: {}", e);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_service,
            commands::stop_service,
            commands::get_service_status,
            commands::refresh_service,
            commands::check_port,
            commands::reclaim_port,
            commands::init_window,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                RunEvent::Exit => {
                    let state: tauri::State<'_, clash::ClashServiceState> = app_handle.state();
                    clash::stop_service_impl(&state);
                }
                RunEvent::ExitRequested { api, code, .. } => {
                    if code.is_none() {
                        api.prevent_exit();
                    }
                }
                _ => {}
            }
        });
}
