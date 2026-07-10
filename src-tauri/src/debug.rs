#[cfg(any(debug_assertions, feature = "devtools"))]
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

/// 打开当前焦点窗口的 DevTools；若无焦点则尝试工具窗口，最后回退主窗口。
#[cfg(any(debug_assertions, feature = "devtools"))]
fn open_webview_devtools<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    for (label, window) in app.webview_windows() {
        if window.is_focused().unwrap_or(false) {
            window.open_devtools();
            log::info!("DevTools opened for focused window: {label}");
            return;
        }
    }

    for label in ["color-picker", "image-editor"] {
        if let Some(window) = app.get_webview_window(label) {
            if window.is_visible().unwrap_or(false) {
                window.open_devtools();
                log::info!("DevTools opened for tool window: {label}");
                return;
            }
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        window.open_devtools();
        log::info!("DevTools opened for main window");
    }
}

#[cfg(not(any(debug_assertions, feature = "devtools")))]
fn open_webview_devtools<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let _ = app;
    log::warn!(
        "DevTools shortcut ignored in release build; rebuild with debug profile or enable tauri `devtools` feature"
    );
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn setup_devtools_shortcut<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

    let devtools_enabled = cfg!(debug_assertions) || env_enabled("VOID_DEVTOOLS");
    if !devtools_enabled {
        return Ok(());
    }

    let mods = Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT;
    let shortcut = Shortcut::new(Some(mods), Code::KeyI);
    let gs = app.global_shortcut();

    gs.on_shortcut(shortcut, |app, _, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        open_webview_devtools(app);
    })?;

    gs.register(shortcut)?;
    if cfg!(debug_assertions) {
        log::info!("DevTools shortcut: Ctrl+Shift+Alt+I (debug build)");
    } else {
        log::info!("DevTools shortcut: Ctrl+Shift+Alt+I (VOID_DEVTOOLS=1)");
    }
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn setup_devtools_shortcut<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
