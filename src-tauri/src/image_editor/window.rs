//! 打开独立图片编辑窗口

use tauri::{AppHandle, Manager, WebviewWindowBuilder};

#[cfg(debug_assertions)]
use tauri::webview::PageLoadEvent;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

use crate::window::{
    present_maximized_tool_window, tool_webview_url, tool_window_placement, WINDOW_BG,
};

const SESSION_ROUTE: &str = "/tool/image-editor/session";

pub fn open_image_editor_window_inner(app: &AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("image-editor") {
        win.destroy()
            .map_err(|e| format!("销毁旧编辑窗口失败: {e}"))?;
    }

    let (logical_w, logical_h, pos_x, pos_y) = tool_window_placement(app)?;

    let base_builder = WebviewWindowBuilder::new(
        app,
        "image-editor",
        tool_webview_url(app, SESSION_ROUTE),
    )
    .title("图片编辑")
    .inner_size(logical_w, logical_h)
    .position(pos_x, pos_y)
    .decorations(true)
    .maximized(true);

    #[cfg(debug_assertions)]
    let base_builder = base_builder.on_page_load(|webview, payload| {
        if payload.event() == PageLoadEvent::Finished {
            webview.open_devtools();
            log::info!("图片编辑窗口 DevTools 已打开");
        }
    });

    #[cfg(target_os = "macos")]
    let builder = base_builder.title_bar_style(TitleBarStyle::Visible);
    #[cfg(not(target_os = "macos"))]
    let builder = base_builder;

    let win = builder
        .resizable(true)
        .background_color(WINDOW_BG)
        .visible(true)
        .build()
        .map_err(|e| format!("创建编辑窗口失败: {e}"))?;

    present_maximized_tool_window(&win);

    Ok(())
}
