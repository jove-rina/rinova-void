#[cfg(any(target_os = "macos", target_os = "windows"))]
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use tauri::Runtime;

/// Register global shortcut: Cmd+Shift+V (macOS) / Ctrl+Shift+V (Windows).
pub fn setup<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        #[cfg(target_os = "macos")]
        let mods = Modifiers::SUPER | Modifiers::SHIFT;
        #[cfg(not(target_os = "macos"))]
        let mods = Modifiers::CONTROL | Modifiers::SHIFT;

        let shortcut = Shortcut::new(Some(mods), Code::KeyV);
        let gs = app.global_shortcut();

        let _ = gs.unregister(shortcut);

        gs.on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                crate::window::toggle_main_window(app);
            }
        })?;

        match gs.register(shortcut) {
            Ok(()) => log::info!("Global shortcut registered: Cmd/Ctrl+Shift+V"),
            Err(e) => log::warn!("全局快捷键: {}", e),
        }
    }

    Ok(())
}
