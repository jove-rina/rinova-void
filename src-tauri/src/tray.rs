use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

use crate::window::{hide_main_window, show_main_window, toggle_main_window};

pub fn setup<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let icon = app.default_window_icon().ok_or("应用图标未找到")?.clone();

    let show_i = MenuItem::with_id(app, "tray-show", "显示窗口", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "tray-hide", "隐藏窗口", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "tray-quit", "退出 Void", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Void — 虚空口袋")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "tray-show" => show_main_window(app),
            "tray-hide" => hide_main_window(app),
            "tray-quit" => {
                let state = app.state::<crate::clash::ClashServiceState>();
                crate::clash::stop_service_blocking(&state);
                app.exit(0);
            }
            _ => {}
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
