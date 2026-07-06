use crate::clash::ClashServiceState;
use crate::window;
use serde_json::Value;
use tauri::State;

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
