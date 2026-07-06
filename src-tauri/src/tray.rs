use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

use crate::window::{open_about, open_tool, toggle_main_window};

pub fn setup<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let icon = app.default_window_icon().ok_or("应用图标未找到")?.clone();

    let toggle_i = MenuItem::with_id(app, "tray-toggle", "显示/隐藏窗口", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let about_i = MenuItem::with_id(app, "tray-about", "关于", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "tray-quit", "退出", true, None::<&str>)?;

    let mut tool_items = Vec::new();
    for tool in crate::tools::TOOLS {
        tool_items.push(MenuItem::with_id(
            app,
            format!("tray-tool-{}", tool.id),
            tool.name,
            true,
            None::<&str>,
        )?);
    }

    let mut menu_refs: Vec<&dyn tauri::menu::IsMenuItem<R>> = vec![&toggle_i, &sep1];
    for item in &tool_items {
        menu_refs.push(item);
    }
    menu_refs.push(&sep2);
    menu_refs.push(&about_i);
    menu_refs.push(&quit_i);

    let menu = Menu::with_items(app, &menu_refs)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Void — 空")
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "tray-toggle" {
                toggle_main_window(app);
            } else if id == "tray-about" {
                open_about(app);
            } else if id == "tray-quit" {
                let state = app.state::<crate::clash::ClashServiceState>();
                crate::clash::stop_service_blocking(&state);
                app.exit(0);
            } else if let Some(tool_id) = id.strip_prefix("tray-tool-") {
                open_tool(app, tool_id);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
