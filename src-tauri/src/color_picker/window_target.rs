//! 取色会话目标窗口：优先 `color-picker`，回退 `main`。

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

pub const PICKER_WINDOW_LABEL: &str = "color-picker";

pub fn picker_webview<R: Runtime>(app: &AppHandle<R>) -> Result<WebviewWindow<R>, String> {
    app.get_webview_window(PICKER_WINDOW_LABEL)
        .or_else(|| app.get_webview_window("main"))
        .ok_or_else(|| "取色窗口未找到".to_string())
}

pub fn has_dedicated_picker_window<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.get_webview_window(PICKER_WINDOW_LABEL).is_some()
}
