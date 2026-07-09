//! 取色会话生命周期：start / refresh / finish / cancel 与 Tauri 命令入口。

use arboard::Clipboard;
use tauri::{AppHandle, Manager, State};

use crate::color_picker::capture::{build_picker_result, capture_snapshot, CaptureSnapshot};
use crate::color_picker::types::{
    FinishPickerResult, MAX_RADIUS, PickerLaunchConfig, PickerSession, PickerState,
    StartPickerResult, rgb_to_hex,
};
use crate::color_picker::window::open_color_picker_window;
use crate::color_picker::window_layout::{layout_picker_window, restore_picker_window, save_pre_picker_layout};
use crate::color_picker::window_target::{has_dedicated_picker_window, PICKER_WINDOW_LABEL};

/// 注册 `PickerState` 到 Tauri 应用状态。
pub fn setup<M: Manager<tauri::Wry>>(app: &M) -> Result<(), String> {
    app.manage(PickerState::default());
    Ok(())
}

pub fn prepare_picker_launch(
    state: State<PickerState>,
    config: PickerLaunchConfig,
) -> Result<(), String> {
    *state.pending_launch.lock().map_err(|e| e.to_string())? = Some(config);
    Ok(())
}

pub fn take_picker_launch(state: State<PickerState>) -> Result<Option<PickerLaunchConfig>, String> {
    let mut pending = state.pending_launch.lock().map_err(|e| e.to_string())?;
    Ok(pending.take())
}

/// 托盘等入口：保留主窗口并打开取色窗口。
pub fn open_picker_tool<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: State<PickerState>,
) -> Result<(), String> {
    *state
        .pending_launch
        .lock()
        .map_err(|e| e.to_string())? = Some(PickerLaunchConfig::tray_default());
    open_color_picker_window(app)?;
    crate::window::show_main_window(app);
    Ok(())
}

/// 非命令上下文打开取色工具（托盘 / 快捷键路由）。
pub fn open_picker_tool_direct<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    let state: State<PickerState> = app.state();
    open_picker_tool(app, state)
}

/// 枚举当前平台可用显示器（供前端下拉选择）。
pub fn list_monitors(app: &AppHandle) -> Result<Vec<crate::color_picker::types::MonitorInfo>, String> {
    crate::color_picker::platform::list_monitors(app)
}

/// 开始取色：截屏 → 布局窗口 → 激活会话。
pub fn start_picker(
    app: AppHandle,
    state: State<PickerState>,
    radius: Option<u8>,
    hide_app: Option<bool>,
    monitor_index: Option<u32>,
    capture_all: Option<bool>,
) -> Result<StartPickerResult, String> {
    let radius_value = radius.unwrap_or(0).min(MAX_RADIUS);
    let hide_app = hide_app.unwrap_or(true);
    let capture_all = capture_all.unwrap_or(false);
    let monitor_index = monitor_index.unwrap_or(0);

    if *state.is_active.lock().map_err(|e| e.to_string())? {
        return Err("取色已在进行中".into());
    }

    save_pre_picker_layout(&app)?;

    let snapshot = match capture_snapshot(&app, hide_app, monitor_index, capture_all) {
        Ok(snapshot) => snapshot,
        Err(e) => {
            #[cfg(target_os = "macos")]
            recover_window_after_picker_failure(&app, &state);
            return Err(e);
        }
    };

    let result = match apply_snapshot(&app, &state, snapshot, radius_value, monitor_index, capture_all)
    {
        Ok(result) => result,
        Err(e) => {
            #[cfg(target_os = "macos")]
            recover_window_after_picker_failure(&app, &state);
            return Err(e);
        }
    };

    {
        let mut active = state.is_active.lock().map_err(|e| e.to_string())?;
        *active = true;
    }

    Ok(result)
}

/// 刷新截屏（切换显示器 / 全部屏幕 / 隐藏 App / 放大镜半径时）。
pub fn refresh_picker(
    app: AppHandle,
    state: State<PickerState>,
    hide_app: Option<bool>,
    monitor_index: Option<u32>,
    capture_all: Option<bool>,
    radius: Option<u8>,
) -> Result<StartPickerResult, String> {
    let hide_app = hide_app.unwrap_or(true);
    let capture_all = capture_all.unwrap_or(false);
    let monitor_index = monitor_index.unwrap_or(0);

    let session_radius = {
        let active = *state.is_active.lock().map_err(|e| e.to_string())?;
        if !active {
            return Err("取色未开始".into());
        }
        state
            .session
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .map(|session| session.radius)
            .ok_or_else(|| "取色会话无效".to_string())?
    };

    let radius_value = radius.unwrap_or(session_radius).min(MAX_RADIUS);
    let snapshot = match capture_snapshot(&app, hide_app, monitor_index, capture_all) {
        Ok(snapshot) => snapshot,
        Err(e) => {
            #[cfg(target_os = "macos")]
            recover_window_after_picker_failure(&app, &state);
            return Err(e);
        }
    };
    match apply_snapshot(&app, &state, snapshot, radius_value, monitor_index, capture_all) {
        Ok(result) => Ok(result),
        Err(e) => {
            #[cfg(target_os = "macos")]
            recover_window_after_picker_failure(&app, &state);
            Err(e)
        }
    }
}

#[cfg(target_os = "macos")]
fn recover_window_after_picker_failure(app: &AppHandle, state: &PickerState) {
    let _ = restore_picker_window(app, state);
    let _ = crate::color_picker::present_window(app);
}

pub fn finish_picker(
    app: AppHandle,
    state: State<PickerState>,
    cancel: bool,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
) -> Result<FinishPickerResult, String> {
    finish_picker_inner(&app, &state, cancel, r, g, b)
}

/// 窗口关闭等场景下的静默取消。
pub fn cancel_picker(app: &AppHandle) -> Result<(), String> {
    let state: State<PickerState> = app.state();
    let active = *state.is_active.lock().map_err(|e| e.to_string())?;
    if active {
        finish_picker_inner(app, &state, true, None, None, None)?;
    }
    Ok(())
}

pub fn is_picker_active<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> bool {
    let Some(state) = app.try_state::<PickerState>() else {
        return false;
    };
    state
        .is_active
        .lock()
        .map(|active| *active)
        .unwrap_or(false)
}

/// 截屏后的公共步骤：编码 PNG、布局窗口、写入会话半径。
fn apply_snapshot(
    app: &AppHandle,
    state: &PickerState,
    snapshot: CaptureSnapshot,
    radius: u8,
    monitor_index: u32,
    capture_all: bool,
) -> Result<StartPickerResult, String> {
    let result = build_picker_result(&snapshot, radius)?;
    layout_picker_window(app, state, monitor_index, capture_all)?;
    *state
        .session
        .lock()
        .map_err(|e| e.to_string())? = Some(PickerSession { radius });
    Ok(result)
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    Clipboard::new()
        .map_err(|e| format!("剪贴板不可用: {e}"))?
        .set_text(text)
        .map_err(|e| format!("复制失败: {e}"))
}

/// 更新会话标志与剪贴板；不含窗口恢复（由 `finish_picker_inner` 负责）。
pub(crate) fn finish_picker_state(
    state: &PickerState,
    cancel: bool,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
) -> Result<FinishPickerResult, String> {
    let mut active = state.is_active.lock().map_err(|e| e.to_string())?;
    if !*active {
        return Ok(FinishPickerResult {
            copied: false,
            hex: None,
        });
    }

    if !cancel && (r.is_none() || g.is_none() || b.is_none()) {
        return Err("缺少颜色分量".into());
    }

    *state.session.lock().map_err(|e| e.to_string())? = None;

    let mut copied = false;
    let mut hex = None;

    if !cancel {
        if let (Some(r), Some(g), Some(b)) = (r, g, b) {
            let h = rgb_to_hex(r, g, b);
            copy_to_clipboard(&h)?;
            copied = true;
            hex = Some(h);
        }
    }

    *active = false;
    Ok(FinishPickerResult { copied, hex })
}

fn finish_picker_inner(
    app: &AppHandle,
    state: &PickerState,
    cancel: bool,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
) -> Result<FinishPickerResult, String> {
    let result = finish_picker_state(state, cancel, r, g, b)?;
    restore_picker_window(app, state)?;
    if has_dedicated_picker_window(app) {
        if let Some(picker) = app.get_webview_window(PICKER_WINDOW_LABEL) {
            let _ = picker.close();
        }
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.set_focus();
    }
    Ok(result)
}
