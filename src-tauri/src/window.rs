use tauri::{Emitter, LogicalSize, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindow};
use tauri::AppHandle;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
use tauri::window::Color;

pub const WINDOW_BG: Color = Color(22, 23, 29, 255);

/// 工具窗口 URL：dev 走 Vite devUrl + hash；release 走 App 入口 + hash。
pub fn tool_webview_url<R: Runtime>(app: &tauri::AppHandle<R>, route: &str) -> WebviewUrl {
    if cfg!(debug_assertions) {
        if let Some(mut dev_url) = app.config().build.dev_url.clone() {
            dev_url.set_fragment(Some(route));
            log::info!("工具窗口 URL: {dev_url}");
            return WebviewUrl::External(dev_url);
        }
    }

    WebviewUrl::App(format!("index.html#{route}").into())
}

/// Windows WebView2 在同步 command / 事件里 `build()` 会死锁，需独立线程创建。
pub fn spawn_tool_window_creation<R, F>(app: &tauri::AppHandle<R>, create: F) -> Result<(), String>
where
    R: Runtime,
    F: FnOnce(&AppHandle<R>) -> Result<(), String> + Send + 'static,
{
    #[cfg(windows)]
    {
        let app = app.clone();
        std::thread::spawn(move || {
            if let Err(e) = create(&app) {
                log::error!("创建工具窗口失败: {e}");
            }
        });
        return Ok(());
    }

    #[cfg(not(windows))]
    create(app)
}

/// 工具窗口默认落点：最大可用显示器的 work area（逻辑坐标 w, h, x, y）。
pub fn tool_window_placement<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<(f64, f64, f64, f64), String> {
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
    Ok((
        (work.size.width as f64 / scale).round().max(800.0),
        (work.size.height as f64 / scale).round().max(600.0),
        work.position.x as f64 / scale,
        work.position.y as f64 / scale,
    ))
}

/// 显示工具窗口并最大化到当前显示器。
pub fn present_maximized_tool_window<R: Runtime>(win: &WebviewWindow<R>) {
    let _ = win.unminimize();
    let _ = win.show();
    let _ = win.maximize();
    let _ = win.set_focus();
}

/// 与 `tauri.conf.json` 中 main 窗口初始逻辑尺寸一致。
pub const DEFAULT_WINDOW_WIDTH: u32 = 400;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 500;

/// 旧版截屏 hide 可能把窗口缩到 1×1 并持久化；低于此阈值视为损坏。
const MIN_VALID_WIDTH: u32 = 120;
const MIN_VALID_HEIGHT: u32 = 120;
/// 旧版曾移到 -30000，会被 window-state 插件记住。
const OFFSCREEN_THRESHOLD: i32 = 8_000;

#[cfg(target_os = "macos")]
fn unhide_app_if_needed<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Err(e) = crate::color_picker::unhide_app_for_window_show(app) {
        log::warn!("解除 App 隐藏: {}", e);
    }
}

#[cfg(not(target_os = "macos"))]
fn unhide_app_if_needed<R: Runtime>(_app: &tauri::AppHandle<R>) {}

fn is_geometry_corrupted<R: Runtime>(window: &WebviewWindow<R>) -> Result<bool, String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let size = window.outer_size().map_err(|e| e.to_string())?;
    Ok(pos.x.abs() > OFFSCREEN_THRESHOLD
        || pos.y.abs() > OFFSCREEN_THRESHOLD
        || size.width < MIN_VALID_WIDTH
        || size.height < MIN_VALID_HEIGHT)
}

/// 将主窗口设为默认逻辑尺寸并居中于主屏 work area（与 tauri.conf 一致）。
pub fn apply_default_window_geometry<R: Runtime>(
    main: &WebviewWindow<R>,
    app: &tauri::AppHandle<R>,
) -> Result<(), String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let monitor = monitors
        .first()
        .ok_or_else(|| "未检测到可用显示器".to_string())?;
    let work = monitor.work_area();
    let scale = monitor.scale_factor();
    let phys_w = ((DEFAULT_WINDOW_WIDTH as f64) * scale).round() as u32;
    let phys_h = ((DEFAULT_WINDOW_HEIGHT as f64) * scale).round() as u32;
    let x = work.position.x + ((work.size.width.saturating_sub(phys_w)) / 2) as i32;
    let y = work.position.y + ((work.size.height.saturating_sub(phys_h)) / 2) as i32;

    main.set_fullscreen(false)
        .map_err(|e| format!("退出全屏失败: {e}"))?;
    // 部分平台在 resizable=false 时 set_size 不生效，先临时允许调整。
    let _ = main.set_resizable(true);
    main.set_size(LogicalSize::new(
        DEFAULT_WINDOW_WIDTH as f64,
        DEFAULT_WINDOW_HEIGHT as f64,
    ))
    .map_err(|e| format!("设置窗口尺寸失败: {e}"))?;
    main.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| format!("设置窗口位置失败: {e}"))?;
    apply_main_window_constraints(main)
}

/// 主面板固定尺寸：禁止拖拽缩放与最大化。
fn apply_main_window_constraints<R: Runtime>(main: &WebviewWindow<R>) -> Result<(), String> {
    let _ = main.unmaximize();
    main.set_maximizable(false)
        .map_err(|e| format!("禁用最大化失败: {e}"))?;
    main.set_resizable(false)
        .map_err(|e| format!("设置窗口不可调整大小失败: {e}"))?;
    Ok(())
}

fn repair_window_geometry_if_needed<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    unhide_app_if_needed(app);

    if crate::color_picker::is_picker_active(app) {
        #[cfg(target_os = "macos")]
        crate::color_picker::present_window(app)?;
        #[cfg(not(target_os = "macos"))]
        if let Some(main) = app.get_webview_window("main") {
            main.show().map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    let Some(main) = app.get_webview_window("main") else {
        return Ok(());
    };

    if !is_geometry_corrupted(&main)? {
        let _ = apply_native_window_chrome(&main);
        return Ok(());
    }

    apply_default_window_geometry(&main, app)?;
    apply_native_window_chrome(&main)
}

/// 主窗口是否处于可见状态（查询失败时视为不可见）。
pub fn is_main_window_visible<R: Runtime>(app: &tauri::AppHandle<R>) -> bool {
    app.get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

/// 将主窗口恢复到应用初始尺寸并居中于主屏 work area，然后显示。
pub fn reset_main_window<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口未找到".to_string())?;

    unhide_app_if_needed(app);
    apply_default_window_geometry(&main, app)?;
    let _ = main.unminimize();
    main.show().map_err(|e| format!("显示窗口失败: {e}"))?;
    let _ = main.set_focus();

    crate::tray::sync_toggle_menu_label(app);
    Ok(())
}

pub fn show_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    let _ = repair_window_geometry_if_needed(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    crate::tray::sync_toggle_menu_label(app);
}

pub fn toggle_main_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                show_main_window(app);
                return;
            }
        }
    }
    crate::tray::sync_toggle_menu_label(app);
}

/// 启动时修复被 window-state 恢复的损坏几何，并显示主窗口。
pub fn repair_main_window_on_launch<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Err(e) = repair_window_geometry_if_needed(app) {
        log::warn!("启动窗口几何修复: {}", e);
    }
    show_main_window(app);
}

/// 显示主窗口并打开独立取色窗口
pub fn open_color_picker<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Err(e) = crate::color_picker::open_picker_tool_direct(app) {
        log::warn!("打开取色窗口: {}", e);
    }
}

/// 显示主窗口并通知前端打开指定工具
pub fn open_tool<R: Runtime>(app: &tauri::AppHandle<R>, tool_id: &str) {
    if tool_id == "color-picker" {
        open_color_picker(app);
        return;
    }
    if let Some(route) = crate::tools::route_for(tool_id) {
        show_main_window(app);
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.emit("open-tool", route);
        }
    }
}

/// 显示主窗口并通知前端打开关于对话框
pub fn open_about<R: Runtime>(app: &tauri::AppHandle<R>) {
    show_main_window(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("open-about", ());
    }
}

/// 强制使用系统原生窗体（覆盖 window-state 可能恢复的旧无边框状态）。
fn apply_native_window_chrome<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    window
        .set_decorations(true)
        .map_err(|e| format!("设置窗口装饰失败: {e}"))?;

    #[cfg(target_os = "macos")]
    window
        .set_title_bar_style(TitleBarStyle::Visible)
        .map_err(|e| format!("设置标题栏样式失败: {e}"))?;

    apply_main_window_constraints(window)
}

/// Apply platform window chrome: solid background matching the frontend theme.
pub fn init_main_window<R: Runtime, M: Manager<R>>(app: &M) -> Result<(), String> {
    let handle = app.app_handle();
    let window = handle
        .get_webview_window("main")
        .ok_or_else(|| "主窗口未找到".to_string())?;

    apply_native_window_chrome(&window)?;

    window
        .set_background_color(Some(WINDOW_BG))
        .map_err(|e| format!("窗口背景色设置失败: {e}"))?;

    Ok(())
}
