//! 大图导出分块缓冲

use std::collections::HashMap;
use std::sync::Mutex;

use base64::Engine;
use tauri::{AppHandle, State};

use crate::export::{convert_image_bytes_to_path, write_binary_to_path};

pub struct ExportBufferState {
    pub buffers: Mutex<HashMap<String, Vec<u8>>>,
}

impl ExportBufferState {
    pub fn new() -> Self {
        Self {
            buffers: Mutex::new(HashMap::new()),
        }
    }
}

fn decode_chunk(chunk_base64: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(chunk_base64.trim())
        .map_err(|e| format!("解码图片数据失败: {e}"))
}

pub fn begin_export_buffer(state: State<ExportBufferState>, export_id: String) -> Result<(), String> {
    let mut map = state.buffers.lock().map_err(|e| e.to_string())?;
    map.insert(export_id, Vec::new());
    Ok(())
}

pub fn append_export_base64(
    state: State<ExportBufferState>,
    export_id: String,
    chunk_base64: String,
) -> Result<(), String> {
    let bytes = decode_chunk(&chunk_base64)?;
    let mut map = state.buffers.lock().map_err(|e| e.to_string())?;
    let buffer = map
        .get_mut(&export_id)
        .ok_or_else(|| "导出会话不存在或已结束".to_string())?;
    buffer.extend_from_slice(&bytes);
    Ok(())
}

pub fn cancel_export_buffer(state: State<ExportBufferState>, export_id: String) -> Result<(), String> {
    let mut map = state.buffers.lock().map_err(|e| e.to_string())?;
    map.remove(&export_id);
    Ok(())
}

pub fn finish_export_binary(
    app: AppHandle,
    state: State<ExportBufferState>,
    export_id: String,
    dest_path: String,
) -> Result<String, String> {
    let _ = app;
    let bytes = {
        let mut map = state.buffers.lock().map_err(|e| e.to_string())?;
        map.remove(&export_id)
            .ok_or_else(|| "导出会话不存在或已结束".to_string())?
    };

    if bytes.is_empty() {
        return Err("导出数据为空".into());
    }

    write_binary_to_path(&dest_path, &bytes)
}

pub fn finish_convert_export(
    app: AppHandle,
    state: State<ExportBufferState>,
    export_id: String,
    dest_path: String,
    format: String,
) -> Result<String, String> {
    let _ = app;
    let bytes = {
        let mut map = state.buffers.lock().map_err(|e| e.to_string())?;
        map.remove(&export_id)
            .ok_or_else(|| "导出会话不存在或已结束".to_string())?
    };

    if bytes.is_empty() {
        return Err("导出数据为空".into());
    }

    convert_image_bytes_to_path(&dest_path, &format, &bytes)
}
