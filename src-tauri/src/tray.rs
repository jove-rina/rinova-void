use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

use crate::window::{is_main_window_visible, open_about, open_tool, reset_main_window, toggle_main_window};

const TOGGLE_SHOW_LABEL: &str = "显示窗口";
const TOGGLE_HIDE_LABEL: &str = "隐藏窗口";

/// 托盘菜单中「显示/隐藏窗口」项，用于动态更新文案。
pub struct TrayMenuState<R: Runtime> {
    toggle_item: MenuItem<R>,
}

impl<R: Runtime> TrayMenuState<R> {
    pub fn sync_toggle_label(&self, app: &tauri::AppHandle<R>) {
        let label = if is_main_window_visible(app) {
            TOGGLE_HIDE_LABEL
        } else {
            TOGGLE_SHOW_LABEL
        };
        if let Err(e) = self.toggle_item.set_text(label) {
            log::warn!("托盘菜单文案更新失败: {}", e);
        }
    }
}

pub fn sync_toggle_menu_label<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(state) = app.try_state::<TrayMenuState<R>>() {
        state.sync_toggle_label(app);
    }
}

pub fn setup<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let icon = app.default_window_icon().ok_or("应用图标未找到")?.clone();

    let toggle_i = MenuItem::with_id(app, "tray-toggle", TOGGLE_SHOW_LABEL, true, None::<&str>)?;
    let reset_i = MenuItem::with_id(app, "tray-reset", "重置窗口", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let about_i = MenuItem::with_id(app, "tray-about", "关于", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "tray-quit", "退出", true, None::<&str>)?;

    app.manage(TrayMenuState {
        toggle_item: toggle_i.clone(),
    });

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

    let mut menu_refs: Vec<&dyn tauri::menu::IsMenuItem<R>> =
        vec![&toggle_i, &reset_i, &sep1];
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
            } else if id == "tray-reset" {
                if let Err(e) = reset_main_window(app) {
                    log::warn!("重置窗口: {}", e);
                }
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
            match &event {
                TrayIconEvent::Enter { .. } | TrayIconEvent::Click { .. } => {
                    sync_toggle_menu_label(tray.app_handle());
                }
                _ => {}
            }

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

    sync_toggle_menu_label(app);

    Ok(())
}
