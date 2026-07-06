use tauri::Manager;

/// 读取环境变量开关（`1` / `true` 开启，`0` / `false` / 未设置 关闭）。
pub fn env_enabled(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let v = v.trim();
            !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
        }
        Err(_) => false,
    }
}

/// Debug 构建或 `VOID_LOG=1` 时写入 OS 日志目录。
pub fn setup_logging<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let enable = cfg!(debug_assertions) || env_enabled("VOID_LOG");
    if !enable {
        return Ok(());
    }

    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Debug
    };

    app.plugin(
        tauri_plugin_log::Builder::default()
            .level(level)
            .target(tauri_plugin_log::Target::new(
                tauri_plugin_log::TargetKind::LogDir {
                    file_name: Some("void".into()),
                },
            ))
            .build(),
    )
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
pub fn setup_devtools_shortcut<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

    if !env_enabled("VOID_DEVTOOLS") {
        return Ok(());
    }

    let mods = Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT;
    let shortcut = Shortcut::new(Some(mods), Code::KeyI);
    let gs = app.global_shortcut();

    gs.on_shortcut(shortcut, |app, _, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        if let Some(w) = app.get_webview_window("main") {
            w.open_devtools();
            log::info!("DevTools opened (VOID_DEVTOOLS=1)");
        }
    })?;

    gs.register(shortcut)?;
    log::info!("DevTools shortcut: Ctrl+Shift+Alt+I (VOID_DEVTOOLS=1)");
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn setup_devtools_shortcut<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
