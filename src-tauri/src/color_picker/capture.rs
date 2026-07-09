//! 截屏编排：枚举显示器、平台截屏、隐藏窗口、组装 `StartPickerResult`。

#[cfg(not(target_os = "macos"))]
use std::thread;
#[cfg(not(target_os = "macos"))]
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::color_picker::platform;
use crate::color_picker::types::{MonitorInfo, ScreenCapture, StartPickerResult};
#[cfg(not(target_os = "macos"))]
use crate::color_picker::window_target::picker_webview;
#[cfg(target_os = "macos")]
use crate::color_picker::types::PickerState;

/// Windows / 其他非 macOS：hide 后等待合成器刷新，再 BitBlt。
#[cfg(not(target_os = "macos"))]
const CAPTURE_HIDE_MS: u64 = 120;

/// macOS 截屏隐藏守卫：Drop 时必定恢复窗口状态（截屏失败也不遗留 hide）。
#[cfg(target_os = "macos")]
struct MacCaptureHideScope<'a> {
    app: &'a AppHandle,
    guard: Option<crate::color_picker::macos::CaptureHideGuard>,
}

#[cfg(target_os = "macos")]
impl Drop for MacCaptureHideScope<'_> {
    fn drop(&mut self) {
        if let Some(guard) = self.guard.take() {
            let _ = crate::color_picker::macos::finish_hide_for_capture(self.app, guard);
        }
    }
}

/// 单次截屏的元数据，供 `build_picker_result` 编码 PNG。
pub(crate) struct CaptureSnapshot {
    pub(crate) capture: ScreenCapture,
    pub(crate) monitor_index: u32,
    pub(crate) monitor_label: String,
    pub(crate) capture_all: bool,
}

/// 执行一次截屏并返回快照。
///
/// - `hide_app`：截屏前暂时隐藏主窗口；macOS 用窗口列表合成并排除本进程窗口。
/// - `capture_all`：true 时拼接虚拟桌面，false 时只截 `monitor_index` 指定屏。
pub fn capture_snapshot(
    app: &AppHandle,
    hide_app: bool,
    monitor_index: u32,
    capture_all: bool,
) -> Result<CaptureSnapshot, String> {
    let monitors = platform::list_monitors(app)?;
    if monitors.is_empty() {
        return Err("未检测到可用显示器".into());
    }

    if !capture_all && monitor_index as usize >= monitors.len() {
        return Err(format!("显示器 {} 不存在", monitor_index));
    }

    #[cfg(target_os = "macos")]
    {
        let picker_active = app
            .try_state::<PickerState>()
            .and_then(|state| state.is_active.lock().ok().map(|active| *active))
            .unwrap_or(false);

        let hide_scope = if hide_app {
            Some(MacCaptureHideScope {
                app,
                guard: Some(crate::color_picker::macos::begin_hide_for_capture(
                    app, picker_active,
                )?),
            })
        } else {
            None
        };

        let snapshot =
            take_platform_snapshot(app, &monitors, monitor_index, capture_all, hide_app)?;

        drop(hide_scope);

        return Ok(snapshot);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let picker = picker_webview(app)?;
        let was_visible = picker.is_visible().unwrap_or(true);
        if hide_app && was_visible {
            picker
                .hide()
                .map_err(|e| format!("隐藏取色窗口失败: {e}"))?;
            thread::sleep(Duration::from_millis(CAPTURE_HIDE_MS));
        }

        let snapshot =
            take_platform_snapshot(app, &monitors, monitor_index, capture_all, hide_app)?;

        if hide_app && was_visible {
            let _ = picker.show();
        }

        Ok(snapshot)
    }
}

fn take_platform_snapshot(
    app: &AppHandle,
    monitors: &[MonitorInfo],
    monitor_index: u32,
    capture_all: bool,
    exclude_own_windows: bool,
) -> Result<CaptureSnapshot, String> {
    if capture_all {
        let capture = platform::capture_virtual_desktop(app, exclude_own_windows)?;
        let label = format!("全部屏幕 · {}×{}", capture.width, capture.height);
        return Ok(CaptureSnapshot {
            capture,
            monitor_index: u32::MAX,
            monitor_label: label,
            capture_all: true,
        });
    }

    let monitor = &monitors[monitor_index as usize];
    let capture = platform::capture_monitor(app, monitor.index, exclude_own_windows)?;
    Ok(CaptureSnapshot {
        capture,
        monitor_index: monitor.index,
        monitor_label: monitor.label.clone(),
        capture_all: false,
    })
}

/// 将截屏快照编码为前端可用的 `StartPickerResult`。
pub fn build_picker_result(
    snapshot: &CaptureSnapshot,
    radius: u8,
) -> Result<StartPickerResult, String> {
    Ok(StartPickerResult {
        image_base64: snapshot.capture.to_png_base64()?,
        width: snapshot.capture.width,
        height: snapshot.capture.height,
        origin_x: snapshot.capture.origin_x,
        origin_y: snapshot.capture.origin_y,
        radius,
        monitor_index: snapshot.monitor_index,
        monitor_label: snapshot.monitor_label.clone(),
        capture_all: snapshot.capture_all,
    })
}
