//! 打开独立图片编辑窗口

use tauri::{AppHandle, Manager, TitleBarStyle, WebviewUrl, WebviewWindowBuilder};

use crate::window::WINDOW_BG;

pub fn open_image_editor_window(app: &AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("image-editor") {
        win.close().map_err(|e| format!("关闭旧编辑窗口失败: {e}"))?;
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

    WebviewWindowBuilder::new(app, "image-editor", WebviewUrl::default())
        .title("图片编辑")
        .inner_size(logical_w, logical_h)
        .position(pos_x, pos_y)
        .decorations(true)
        .title_bar_style(TitleBarStyle::Visible)
        .resizable(true)
        .background_color(WINDOW_BG)
        .build()
        .map_err(|e| format!("创建编辑窗口失败: {e}"))?;

    Ok(())
}
