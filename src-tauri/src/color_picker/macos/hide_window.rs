//! macOS 截屏前隐藏主窗口（不移动窗口、不 hide 整个 App，避免 window-state 持久化屏外坐标）。

use objc2_app_kit::{NSApplication, NSWindow, NSWindowSharingType};
use objc2_foundation::MainThreadMarker;
use tauri::{PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

/// 保存截屏前的窗口状态，供 [`restore_screen_capture`] 还原。
pub struct CaptureHideGuard {
    previous_sharing: NSWindowSharingType,
    previous_position: PhysicalPosition<i32>,
    previous_size: PhysicalSize<u32>,
}

/// 截屏前隐藏主窗口（sharingType + orderOut + Tauri hide）。
pub fn exclude_from_screen_capture(window: &WebviewWindow) -> Result<CaptureHideGuard, String> {
    let previous_position = window.outer_position().map_err(|e| e.to_string())?;
    let previous_size = window.outer_size().map_err(|e| e.to_string())?;

    let ns_ptr = window.ns_window().map_err(|e| e.to_string())?;
    let previous_sharing = unsafe {
        let ns_window: &NSWindow = &*ns_ptr.cast();
        let previous = ns_window.sharingType();
        ns_window.setSharingType(NSWindowSharingType::None);
        ns_window.orderOut(None);
        previous
    };

    window.hide().map_err(|e| format!("隐藏主窗口失败: {e}"))?;

    Ok(CaptureHideGuard {
        previous_sharing,
        previous_position,
        previous_size,
    })
}

/// 还原 sharingType 与窗口几何（仍保持 hidden，由 layout / present 负责显示）。
pub fn restore_screen_capture(
    window: &WebviewWindow,
    guard: CaptureHideGuard,
) -> Result<(), String> {
    let ns_ptr = window.ns_window().map_err(|e| e.to_string())?;
    unsafe {
        let ns_window: &NSWindow = &*ns_ptr.cast();
        ns_window.setSharingType(guard.previous_sharing);
    }
    let _ = window.set_size(guard.previous_size);
    let _ = window.set_position(guard.previous_position);
    Ok(())
}

/// 若 App 被意外 hide（历史版本遗留），先 unhide 再显示窗口。
pub fn unhide_application_if_needed() -> Result<(), String> {
    let mtm = MainThreadMarker::new().ok_or_else(|| "必须在主线程显示窗口".to_string())?;
    let ns_app = NSApplication::sharedApplication(mtm);
    if ns_app.isHidden() {
        ns_app.unhideWithoutActivation();
    }
    Ok(())
}

/// 在主线程将窗口带回前台（orderFront + show + focus）。调用方须已在 AppKit 主线程。
pub fn present_window_on_main_thread<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    unhide_application_if_needed()?;
    let ns_ptr = window.ns_window().map_err(|e| e.to_string())?;
    unsafe {
        let ns_window: &NSWindow = &*ns_ptr.cast();
        ns_window.orderFrontRegardless();
    }
    window.show().map_err(|e| format!("显示主窗口失败: {e}"))?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}
