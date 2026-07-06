use crate::clash::ClashServiceState;
use crate::color_picker::{self, FinishPickerResult, MonitorInfo, PickerState, StartPickerResult};
use crate::window;
use serde_json::Value;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_service(
    url: String,
    port: u16,
    allow_fallback: Option<bool>,
    state: State<'_, ClashServiceState>,
) -> Result<crate::clash::StartServiceResult, String> {
    crate::clash::start_service(&state, url, port, allow_fallback.unwrap_or(false)).await
}

#[tauri::command]
pub fn check_port(port: u16) -> Result<crate::clash::PortCheckResult, String> {
    crate::clash::check_port(port)
}

#[tauri::command]
pub fn reclaim_port(port: u16) -> Result<(), String> {
    crate::clash::reclaim_port(port)
}

#[tauri::command]
pub async fn stop_service(state: State<'_, ClashServiceState>) -> Result<String, String> {
    crate::clash::stop_service_impl(&state).await;
    Ok("服务已停止".to_string())
}

#[tauri::command]
pub fn get_service_status(state: State<'_, ClashServiceState>) -> crate::clash::ServiceStatus {
    crate::clash::get_service_status(&state)
}

#[tauri::command]
pub fn refresh_service(state: State<'_, ClashServiceState>) -> Result<Value, String> {
    crate::clash::refresh_service(&state)
}

#[tauri::command]
pub fn init_window(app: tauri::AppHandle) -> Result<(), String> {
    window::init_main_window(&app)
}

#[tauri::command]
pub fn list_picker_monitors() -> Result<Vec<MonitorInfo>, String> {
    color_picker::list_monitors()
}

#[tauri::command]
pub fn start_picker(
    app: AppHandle,
    state: State<PickerState>,
    radius: Option<u8>,
    hide_app: Option<bool>,
    monitor_index: Option<u32>,
    capture_all: Option<bool>,
) -> Result<StartPickerResult, String> {
    color_picker::start_picker(app, state, radius, hide_app, monitor_index, capture_all)
}

#[tauri::command]
pub fn refresh_picker(
    app: AppHandle,
    state: State<PickerState>,
    hide_app: Option<bool>,
    monitor_index: Option<u32>,
    capture_all: Option<bool>,
    radius: Option<u8>,
) -> Result<StartPickerResult, String> {
    color_picker::refresh_picker(app, state, hide_app, monitor_index, capture_all, radius)
}

#[tauri::command]
pub fn finish_picker(
    app: AppHandle,
    state: State<PickerState>,
    cancel: bool,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
) -> Result<FinishPickerResult, String> {
    color_picker::finish_picker(app, state, cancel, r, g, b)
}

#[tauri::command]
pub fn export_text_file(
    app: AppHandle,
    filename: String,
    content: String,
) -> Result<String, String> {
    crate::export::export_text_to_downloads(&app, &filename, &content)
}

#[tauri::command]
pub fn reveal_export_path(path: String) -> Result<(), String> {
    crate::export::reveal_export_path(&path)
}
