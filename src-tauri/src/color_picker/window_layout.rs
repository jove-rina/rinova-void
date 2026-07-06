//! 取色会话窗口布局。
//!
//! - **macOS**：后端将主窗口铺满 work area（排除菜单栏 / Dock），结束时恢复几何。
//! - **Windows / 其他**：由前端 `setFullscreen(true)` 全屏，结束时 `restore_picker_window` 退出全屏。

#[cfg(target_os = "macos")]
pub use crate::color_picker::macos::layout::{
    layout_picker_window, restore_picker_window, save_pre_picker_layout,
};

#[cfg(not(target_os = "macos"))]
use tauri::AppHandle;

#[cfg(not(target_os = "macos"))]
use crate::color_picker::types::PickerState;

#[cfg(not(target_os = "macos"))]
pub fn save_pre_picker_layout(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn layout_picker_window(
    _app: &AppHandle,
    _state: &PickerState,
    _monitor_index: u32,
    _capture_all: bool,
) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn restore_picker_window(app: &AppHandle, _state: &PickerState) -> Result<(), String> {
    let Some(main) = app.get_webview_window("main") else {
        return Ok(());
    };
    let _ = main.set_fullscreen(false);
    Ok(())
}
