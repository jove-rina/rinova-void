//! 图片编辑器会话载荷（主窗口 → 编辑窗口）

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReadImageFileResult {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorImageItem {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
    pub export_name: String,
    pub edit_state: Option<Value>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSessionMeta {
    pub active_id: String,
    pub export_mode: String,
    pub export_format_id: String,
    pub compression_preset_id: String,
    pub size_scale_preset_id: String,
    pub thumbnail_sizes: Vec<u32>,
    pub ico_sizes: Vec<u32>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSessionBatch {
    pub meta: EditorSessionMeta,
    pub images: Vec<EditorImageItem>,
}

pub struct ImageEditorState {
    pub staging: Mutex<HashMap<String, (EditorSessionMeta, Vec<EditorImageItem>)>>,
    pub pending: Mutex<Option<EditorSessionBatch>>,
}

impl ImageEditorState {
    pub fn new() -> Self {
        Self {
            staging: Mutex::new(HashMap::new()),
            pending: Mutex::new(None),
        }
    }
}

const IMAGE_EXT: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "ico", "avif", "svg",
];

fn is_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            IMAGE_EXT.iter().any(|candidate| *candidate == lower)
        })
        .unwrap_or(false)
}

pub fn read_image_file(path: String) -> Result<ReadImageFileResult, String> {
    let path = Path::new(&path);
    if !path.is_file() {
        return Err("文件不存在".into());
    }
    if !is_image_path(path) {
        return Err("不支持的图片格式".into());
    }

    let bytes = std::fs::read(path).map_err(|e| format!("读取文件失败: {e}"))?;
    let img = image::load_from_memory(&bytes).map_err(|e| format!("无法解析图片: {e}"))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image")
        .to_string();

    Ok(ReadImageFileResult {
        name,
        width: img.width(),
        height: img.height(),
        bytes,
    })
}

pub fn begin_editor_session(
    state: State<ImageEditorState>,
    session_id: String,
    meta: EditorSessionMeta,
) -> Result<(), String> {
    if session_id.trim().is_empty() {
        return Err("会话 ID 无效".into());
    }
    if meta.active_id.trim().is_empty() {
        return Err("未指定活动图片".into());
    }
    let mut staging = state.staging.lock().map_err(|e| e.to_string())?;
    staging.insert(session_id, (meta, Vec::new()));
    Ok(())
}

pub fn append_editor_session_image(
    state: State<ImageEditorState>,
    session_id: String,
    item: EditorImageItem,
) -> Result<(), String> {
    if item.bytes.is_empty() {
        return Err("图片数据为空".into());
    }
    let mut staging = state.staging.lock().map_err(|e| e.to_string())?;
    let entry = staging
        .get_mut(&session_id)
        .ok_or_else(|| "编辑会话不存在或已过期".to_string())?;
    entry.1.push(item);
    Ok(())
}

pub fn commit_editor_session(
    state: State<ImageEditorState>,
    session_id: String,
) -> Result<(), String> {
    let mut staging = state.staging.lock().map_err(|e| e.to_string())?;
    let (meta, images) = staging
        .remove(&session_id)
        .ok_or_else(|| "编辑会话不存在或已过期".to_string())?;
    if images.is_empty() {
        return Err("请至少添加一张图片".into());
    }
    let mut pending = state.pending.lock().map_err(|e| e.to_string())?;
    *pending = Some(EditorSessionBatch { meta, images });
    Ok(())
}

pub fn take_image_editor_session(
    state: State<ImageEditorState>,
) -> Result<Option<EditorSessionBatch>, String> {
    let mut pending = state.pending.lock().map_err(|e| e.to_string())?;
    Ok(pending.take())
}
