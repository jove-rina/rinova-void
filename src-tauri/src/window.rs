use tauri::{Emitter, Manager, Runtime};
use tauri::window::Color;

pub const WINDOW_BG: Color = Color(22, 23, 29, 255);

pub fn show_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn toggle_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}

/// 显示主窗口并通知前端进入取色流程
pub fn open_color_picker<R: Runtime>(app: &tauri::AppHandle<R>) {
    show_main_window(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("open-color-picker", ());
    }
}

/// 显示主窗口并通知前端打开指定工具
pub fn open_tool<R: Runtime>(app: &tauri::AppHandle<R>, tool_id: &str) {
    if tool_id == "color-picker" {
        open_color_picker(app);
        return;
    }
    if let Some(route) = crate::tools::route_for(tool_id) {
        show_main_window(app);
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.emit("open-tool", route);
        }
    }
}

/// 显示主窗口并通知前端打开关于对话框
pub fn open_about<R: Runtime>(app: &tauri::AppHandle<R>) {
    show_main_window(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("open-about", ());
    }
}

/// Apply platform window chrome: solid background matching the frontend theme.
pub fn init_main_window<R: Runtime, M: Manager<R>>(app: &M) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口未找到".to_string())?;

    window
        .set_background_color(Some(WINDOW_BG))
        .map_err(|e| format!("窗口背景色设置失败: {e}"))?;

    Ok(())
}
