//! 打开独立取色窗口

use tauri::{Manager, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

use crate::window::WINDOW_BG;

pub fn open_color_picker_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(super::window_target::PICKER_WINDOW_LABEL) {
        let _ = win.unminimize();
        win.show().map_err(|e| format!("显示取色窗口失败: {e}"))?;
        let _ = win.set_focus();
        return Ok(());
    }

    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let monitor = monitors
        .iter()
        .max_by_key(|m| {
            let area = m.work_area();
            area.size.width.saturating_mul(area.size.height)
        })
        .or(monitors.first())
        .ok_or_else(|| "未检测到可用显示器".to_string())?;

    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    let logical_w = (work.size.width as f64 / scale).round().max(800.0);
    let logical_h = (work.size.height as f64 / scale).round().max(600.0);
    let pos_x = work.position.x as f64 / scale;
    let pos_y = work.position.y as f64 / scale;

    WebviewWindowBuilder::new(
        app,
        super::window_target::PICKER_WINDOW_LABEL,
        WebviewUrl::default(),
    )
    .title("取色")
    .inner_size(logical_w, logical_h)
    .position(pos_x, pos_y)
    .decorations(true)
    .title_bar_style(TitleBarStyle::Visible)
    .resizable(true)
    .background_color(WINDOW_BG)
    .build()
    .map_err(|e| format!("创建取色窗口失败: {e}"))?;

    Ok(())
}
