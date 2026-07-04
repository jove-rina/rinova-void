use tauri::{Manager, Runtime};

pub fn show_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn hide_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
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

/// Apply platform-specific rounded window effects (macOS) and enable transparency support.
pub fn init_main_window<R: Runtime, M: Manager<R>>(app: &M) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口未找到".to_string())?;

    #[cfg(target_os = "macos")]
    {
        use tauri::window::{Effect, EffectState, EffectsBuilder};

        window
            .set_effects(
                EffectsBuilder::new()
                    .effect(Effect::HudWindow)
                    .state(EffectState::FollowsWindowActiveState)
                    .radius(24.0)
                    .build(),
            )
            .map_err(|e| format!("窗口效果设置失败: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        let _ = window;
    }

    Ok(())
}
