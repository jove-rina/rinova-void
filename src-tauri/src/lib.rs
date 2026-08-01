mod image_editor;
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
use tauri_plugin_window_state::StateFlags;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_window_state::Builder::new()
                // 不持久化 decorations，避免从旧版无边框状态恢复
                .with_state_flags(
                    StateFlags::SIZE
                        | StateFlags::POSITION
                        | StateFlags::MAXIMIZED
                        | StateFlags::VISIBLE
                        | StateFlags::FULLSCREEN,
                )
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            debug::setup_logging(app.handle())?;
            clash::setup(app)?;
            if let Err(e) = color_picker::setup(app) {
                log::warn!("取色器: {}", e);
            }
            app.manage(image_editor::ImageEditorState::new());
            app.manage(image_editor::ExportBufferState::new());
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
                let app = window.app_handle();
                let label = window.label();

                if label == "color-picker" || label == "image-editor" {
                    if color_picker::is_picker_active(app) && label == "color-picker" {
                        if let Err(e) = color_picker::cancel_picker(app) {
                            log::warn!("取色取消: {}", e);
                        }
                    }
                    return;
                }

                if label != "main" {
                    return;
                }

                if color_picker::is_picker_active(app)
                    && !color_picker::has_dedicated_picker_window(app)
                {
                    api.prevent_close();
                    if let Err(e) = color_picker::cancel_picker(app) {
                        log::warn!("取色取消: {}", e);
                    }
                    return;
                }

                let _ = window.hide();
                crate::tray::sync_toggle_menu_label(app);
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
            commands::reset_window,
            commands::list_picker_monitors,
            commands::prepare_picker_launch,
            commands::take_picker_launch,
            commands::open_color_picker_window,
            commands::start_picker,
            commands::refresh_picker,
            commands::finish_picker,
            commands::export_text_file,
            commands::export_binary_file,
            commands::export_binary_file_base64,
            commands::export_binary_base64_to_path,
            commands::convert_image_base64_to_path,
            commands::export_rgba_image,
            commands::convert_image_base64,
            commands::read_image_file,
            commands::begin_editor_session,
            commands::append_editor_session_image,
            commands::commit_editor_session,
            commands::take_image_editor_session,
            commands::save_image_editor_project,
            commands::list_image_editor_projects,
            commands::load_image_editor_project,
            commands::delete_image_editor_project,
            commands::open_image_editor_window,
            commands::begin_export_buffer,
            commands::append_export_base64,
            commands::cancel_export_buffer,
            commands::finish_export_binary,
            commands::finish_convert_export,
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
