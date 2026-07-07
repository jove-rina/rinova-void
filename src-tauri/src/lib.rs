mod clash;
mod color_picker;
mod commands;
mod debug;
mod export;
mod shortcut;
mod tools;
mod tray;
mod window;

use tauri::{Manager, RunEvent, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            debug::setup_logging(app.handle())?;
            clash::setup(app)?;
            if let Err(e) = color_picker::setup(app) {
                log::warn!("取色器: {}", e);
            }
            if let Err(e) = window::init_main_window(app.handle()) {
                log::warn!("窗口初始化: {}", e);
            }
            if let Err(e) = tray::setup(app.handle()) {
                log::warn!("系统托盘: {}", e);
            }
            window::repair_main_window_on_launch(app.handle());
            if let Err(e) = shortcut::setup(app.handle()) {
                log::warn!("全局快捷键: {}", e);
            }
            if let Err(e) = debug::setup_devtools_shortcut(app.handle()) {
                log::warn!("DevTools 快捷键: {}", e);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" && color_picker::is_picker_active(window.app_handle()) {
                    api.prevent_close();
                    let app = window.app_handle().clone();
                    if let Err(e) = color_picker::cancel_picker(&app) {
                        log::warn!("取色取消: {}", e);
                    }
                    return;
                }
                let _ = window.hide();
                crate::tray::sync_toggle_menu_label(window.app_handle());
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
            commands::list_picker_monitors,
            commands::start_picker,
            commands::refresh_picker,
            commands::finish_picker,
            commands::export_text_file,
            commands::export_binary_file,
            commands::export_rgba_image,
            commands::reveal_export_path,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            match event {
                RunEvent::Exit => {
                    let _ = color_picker::cancel_picker(app_handle);
                    let state: tauri::State<'_, clash::ClashServiceState> = app_handle.state();
                    clash::stop_service_blocking(&state);
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
