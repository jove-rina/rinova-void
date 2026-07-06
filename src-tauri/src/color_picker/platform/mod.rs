//! 平台截屏分发：macOS → [`macos`]，Windows → GDI，其余 → [`unsupported`]。

#[cfg(windows)]
mod windows;
#[cfg(not(any(windows, target_os = "macos")))]
mod unsupported;

use tauri::AppHandle;

use crate::color_picker::types::{MonitorInfo, ScreenCapture};

pub fn list_monitors(app: &AppHandle) -> Result<Vec<MonitorInfo>, String> {
    #[cfg(target_os = "macos")]
    {
        return crate::color_picker::macos::list_monitors(app);
    }
    #[cfg(windows)]
    {
        let _ = app;
        return windows::list_monitors();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = app;
        unsupported::list_monitors()
    }
}

pub fn capture_monitor(
    app: &AppHandle,
    index: u32,
    exclude_own_windows: bool,
) -> Result<ScreenCapture, String> {
    #[cfg(target_os = "macos")]
    {
        return crate::color_picker::macos::capture_monitor(app, index, exclude_own_windows);
    }
    #[cfg(windows)]
    {
        let _ = (app, exclude_own_windows);
        return windows::capture_monitor(index);
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (app, exclude_own_windows);
        unsupported::capture_monitor(index)
    }
}

pub fn capture_virtual_desktop(
    app: &AppHandle,
    exclude_own_windows: bool,
) -> Result<ScreenCapture, String> {
    #[cfg(target_os = "macos")]
    {
        return crate::color_picker::macos::capture_virtual_desktop(app, exclude_own_windows);
    }
    #[cfg(windows)]
    {
        let _ = (app, exclude_own_windows);
        return windows::capture_virtual_desktop();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (app, exclude_own_windows);
        unsupported::capture_virtual_desktop()
    }
}
