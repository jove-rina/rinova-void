//! 取色器 — 跨平台会话、截屏与窗口布局。
//!
//! 模块划分：
//! - [`session`] — Tauri 命令入口与会话状态机
//! - [`capture`] — 截屏编排与 PNG 编码
//! - [`platform`] — Windows GDI / macOS CGDisplay 等平台实现
//! - [`window_layout`] — macOS work area 窗口布局（其他平台 no-op）

mod capture;
mod platform;
mod session;
mod types;
mod window_layout;

#[cfg(target_os = "macos")]
mod macos;

pub use session::{
    cancel_picker, finish_picker, is_picker_active, list_monitors, refresh_picker, setup,
    start_picker,
};
pub use types::{FinishPickerResult, MonitorInfo, PickerState, StartPickerResult};

#[cfg(target_os = "macos")]
pub fn unhide_app_for_window_show<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    let app = app.clone();
    macos::run_on_main_thread(app, || macos::unhide_application_if_needed())
}

#[cfg(target_os = "macos")]
pub fn present_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), String> {
    macos::present_main_window(app)
}

#[cfg(test)]
mod tests {
    use crate::color_picker::session::finish_picker_state;
    use crate::color_picker::types::{PickerSession, PickerState, rgb_to_hex};

    fn active_state() -> PickerState {
        PickerState {
            is_active: std::sync::Mutex::new(true),
            session: std::sync::Mutex::new(Some(PickerSession { radius: 0 })),
            #[cfg(target_os = "macos")]
            saved_layout: std::sync::Mutex::new(None),
        }
    }

    #[test]
    fn rgb_to_hex_formats() {
        assert_eq!(rgb_to_hex(192, 132, 252), "#c084fc");
        assert_eq!(rgb_to_hex(0, 0, 0), "#000000");
        assert_eq!(rgb_to_hex(255, 255, 255), "#ffffff");
    }

    #[test]
    fn finish_inactive_returns_not_copied() {
        let state = PickerState::default();
        let result = finish_picker_state(&state, true, None, None, None).unwrap();
        assert!(!result.copied);
        assert!(result.hex.is_none());
    }

    #[test]
    fn finish_cancel_clears_session_without_copy() {
        let state = active_state();
        let result = finish_picker_state(&state, true, None, None, None).unwrap();
        assert!(!result.copied);
        assert!(result.hex.is_none());
        assert!(!*state.is_active.lock().unwrap());
        assert!(state.session.lock().unwrap().is_none());
    }

    #[test]
    fn finish_confirm_requires_rgb_components() {
        let state = active_state();
        let err = finish_picker_state(&state, false, None, None, None).unwrap_err();
        assert_eq!(err, "缺少颜色分量");
        assert!(*state.is_active.lock().unwrap());
        assert!(state.session.lock().unwrap().is_some());
    }
}
