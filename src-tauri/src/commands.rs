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
pub fn list_picker_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    color_picker::list_monitors(&app)
}

#[tauri::command]
pub fn prepare_picker_launch(
    state: State<PickerState>,
    config: color_picker::PickerLaunchConfig,
) -> Result<(), String> {
    color_picker::prepare_picker_launch(state, config)
}

#[tauri::command]
pub fn take_picker_launch(
    state: State<PickerState>,
) -> Result<Option<color_picker::PickerLaunchConfig>, String> {
    color_picker::take_picker_launch(state)
}

#[tauri::command]
pub async fn open_color_picker_window(app: AppHandle) -> Result<(), String> {
    color_picker::open_color_picker_window_inner(&app)
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
pub fn export_binary_file(
    app: AppHandle,
    filename: String,
    content: Vec<u8>,
) -> Result<String, String> {
    crate::export::export_binary_to_downloads(&app, &filename, &content)
}

#[tauri::command]
pub fn export_binary_file_base64(
    app: AppHandle,
    filename: String,
    content_base64: String,
) -> Result<String, String> {
    crate::export::export_binary_base64_to_downloads(&app, &filename, &content_base64)
}

#[tauri::command]
pub fn export_binary_base64_to_path(
    dest_path: String,
    content_base64: String,
) -> Result<String, String> {
    crate::export::export_binary_base64_to_path(&dest_path, &content_base64)
}

#[tauri::command]
pub fn convert_image_base64_to_path(
    dest_path: String,
    format: String,
    content_base64: String,
) -> Result<String, String> {
    crate::export::convert_image_base64_to_path(&dest_path, &format, &content_base64)
}

#[tauri::command]
pub fn export_rgba_image(
    app: AppHandle,
    filename: String,
    format: String,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
) -> Result<String, String> {
    crate::export::export_rgba_image(&app, &filename, &format, width, height, rgba)
}

#[tauri::command]
pub fn convert_image_base64(
    app: AppHandle,
    filename: String,
    format: String,
    content_base64: String,
) -> Result<String, String> {
    crate::export::convert_image_base64(&app, &filename, &format, &content_base64)
}

#[tauri::command]
pub fn read_image_file(path: String) -> Result<crate::image_editor::ReadImageFileResult, String> {
    crate::image_editor::read_image_file(path)
}

#[tauri::command]
pub fn begin_editor_session(
    state: tauri::State<crate::image_editor::ImageEditorState>,
    session_id: String,
    meta: crate::image_editor::EditorSessionMeta,
) -> Result<(), String> {
    crate::image_editor::begin_editor_session(state, session_id, meta)
}

#[tauri::command]
pub fn append_editor_session_image(
    state: tauri::State<crate::image_editor::ImageEditorState>,
    session_id: String,
    item: crate::image_editor::EditorImageItem,
) -> Result<(), String> {
    crate::image_editor::append_editor_session_image(state, session_id, item)
}

#[tauri::command]
pub fn commit_editor_session(
    state: tauri::State<crate::image_editor::ImageEditorState>,
    session_id: String,
) -> Result<(), String> {
    crate::image_editor::commit_editor_session(state, session_id)
}

#[tauri::command]
pub fn take_image_editor_session(
    state: tauri::State<crate::image_editor::ImageEditorState>,
) -> Result<Option<crate::image_editor::EditorSessionBatch>, String> {
    crate::image_editor::take_image_editor_session(state)
}

#[tauri::command]
pub fn save_image_editor_project(app: AppHandle, json: String) -> Result<String, String> {
    crate::image_editor::save_image_editor_project(&app, json)
}

#[tauri::command]
pub fn list_image_editor_projects(
    app: AppHandle,
) -> Result<Vec<crate::image_editor::ProjectSummary>, String> {
    crate::image_editor::list_image_editor_projects(&app)
}

#[tauri::command]
pub fn load_image_editor_project(app: AppHandle, id: String) -> Result<String, String> {
    crate::image_editor::load_image_editor_project(&app, id)
}

#[tauri::command]
pub fn delete_image_editor_project(app: AppHandle, id: String) -> Result<(), String> {
    crate::image_editor::delete_image_editor_project(&app, id)
}

#[tauri::command]
pub async fn open_image_editor_window(app: AppHandle) -> Result<(), String> {
    crate::image_editor::open_image_editor_window_inner(&app)
}

#[tauri::command]
pub fn begin_export_buffer(
    state: State<crate::image_editor::ExportBufferState>,
    export_id: String,
) -> Result<(), String> {
    crate::image_editor::begin_export_buffer(state, export_id)
}

#[tauri::command]
pub fn append_export_base64(
    state: State<crate::image_editor::ExportBufferState>,
    export_id: String,
    chunk_base64: String,
) -> Result<(), String> {
    crate::image_editor::append_export_base64(state, export_id, chunk_base64)
}

#[tauri::command]
pub fn cancel_export_buffer(
    state: State<crate::image_editor::ExportBufferState>,
    export_id: String,
) -> Result<(), String> {
    crate::image_editor::cancel_export_buffer(state, export_id)
}

#[tauri::command]
pub fn finish_export_binary(
    app: AppHandle,
    state: State<crate::image_editor::ExportBufferState>,
    export_id: String,
    dest_path: String,
) -> Result<String, String> {
    crate::image_editor::finish_export_binary(app, state, export_id, dest_path)
}

#[tauri::command]
pub fn finish_convert_export(
    app: AppHandle,
    state: State<crate::image_editor::ExportBufferState>,
    export_id: String,
    dest_path: String,
    format: String,
) -> Result<String, String> {
    crate::image_editor::finish_convert_export(app, state, export_id, dest_path, format)
}

#[tauri::command]
pub fn reveal_export_path(path: String) -> Result<(), String> {
    crate::export::reveal_export_path(&path)
}
