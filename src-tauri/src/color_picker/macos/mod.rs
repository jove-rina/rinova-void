//! macOS 取色专用：CGDisplay 截屏、截屏前隐藏窗口、work area 布局。

mod capture;
mod hide_window;
pub mod layout;
mod screen_access;

pub use capture::{capture_monitor, capture_virtual_desktop, list_monitors};
pub use hide_window::{unhide_application_if_needed, CaptureHideGuard};

use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager};

/// 首次截屏：等待窗口 hide 生效。
const CAPTURE_HIDE_MS: u64 = 350;
/// 会话内 refresh：窗口可能刚恢复，需更长等待。
const CAPTURE_REFRESH_HIDE_MS: u64 = 550;
/// 主线程 RunLoop 额外等待，确保 compositor 刷新完成。
const CAPTURE_COMPOSITOR_FLUSH_S: f64 = 0.12;

fn flush_compositor_on_main_thread(app: &AppHandle) -> Result<(), String> {
    run_on_main_thread(app.clone(), || {
        use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoopRunInMode};
        unsafe {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, CAPTURE_COMPOSITOR_FLUSH_S, 0);
        }
        Ok(())
    })
}

/// 将闭包派发到 AppKit 主线程并同步等待结果（NSWindow API 必须在主线程调用）。
pub fn run_on_main_thread<Rt, R, F>(app: AppHandle<Rt>, f: F) -> Result<R, String>
where
    Rt: tauri::Runtime,
    R: Send + 'static,
    F: FnOnce() -> Result<R, String> + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    app.run_on_main_thread(move || {
        let _ = tx.send(f());
    })
    .map_err(|e| format!("主线程调度失败: {e}"))?;
    rx.recv()
        .map_err(|_| String::from("主线程操作未完成"))?
}

/// 截屏前隐藏应用；返回 guard，截屏结束后须调用 [`finish_hide_for_capture`]。
pub fn begin_hide_for_capture(app: &AppHandle, picker_active: bool) -> Result<CaptureHideGuard, String> {
    let app_for_hide = app.clone();
    let guard = run_on_main_thread(app.clone(), move || {
        let main = app_for_hide
            .get_webview_window("main")
            .ok_or_else(|| "主窗口未找到".to_string())?;
        hide_window::exclude_from_screen_capture(&main)
    })?;
    let hide_ms = if picker_active {
        CAPTURE_REFRESH_HIDE_MS
    } else {
        CAPTURE_HIDE_MS
    };
    thread::sleep(Duration::from_millis(hide_ms));
    flush_compositor_on_main_thread(app)?;
    Ok(guard)
}

pub fn finish_hide_for_capture(app: &AppHandle, guard: CaptureHideGuard) -> Result<(), String> {
    let app_for_restore = app.clone();
    run_on_main_thread(app.clone(), move || {
        let main = app_for_restore
            .get_webview_window("main")
            .ok_or_else(|| "主窗口未找到".to_string())?;
        hide_window::restore_screen_capture(&main, guard)
    })
}

/// 从任意线程显示主窗口（托盘菜单等场景须走主线程派发）。
pub fn present_main_window<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let app = app.clone();
    run_on_main_thread(app.clone(), move || {
        let main = app
            .get_webview_window("main")
            .ok_or_else(|| "主窗口未找到".to_string())?;
        hide_window::present_window_on_main_thread(&main)
    })
}
