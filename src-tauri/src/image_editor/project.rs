//! 图片编辑项目持久化（app_data/image-editor/projects）

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub updated_at: u64,
    pub image_count: u32,
    pub preview_base64: Option<String>,
}

fn projects_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {e}"))?
        .join("image-editor")
        .join("projects");
    fs::create_dir_all(&dir).map_err(|e| format!("创建项目目录失败: {e}"))?;
    Ok(dir)
}

fn project_path(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    let safe = id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>();
    if safe.is_empty() || safe != id {
        return Err("项目 ID 无效".into());
    }
    Ok(projects_dir(app)?.join(format!("{safe}.json")))
}

pub fn save_image_editor_project(app: &AppHandle, json: String) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| format!("项目数据无效: {e}"))?;
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "项目缺少 id".to_string())?
        .to_string();
    let path = project_path(app, &id)?;
    fs::write(&path, json.as_bytes()).map_err(|e| format!("写入项目失败: {e}"))?;
    Ok(id)
}

pub fn list_image_editor_projects(app: &AppHandle) -> Result<Vec<ProjectSummary>, String> {
    let dir = projects_dir(app)?;
    let mut items = Vec::new();

    for entry in fs::read_dir(&dir).map_err(|e| format!("读取项目目录失败: {e}"))? {
        let entry = entry.map_err(|e| format!("读取项目条目失败: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path).map_err(|e| format!("读取项目失败: {e}"))?;
        let value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|e| format!("解析项目失败: {e}"))?;
        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            continue;
        }
        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("未命名项目")
            .to_string();
        let updated_at = value
            .get("updatedAt")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let image_count = value
            .get("documents")
            .and_then(|v| v.as_array())
            .map(|a| a.len() as u32)
            .unwrap_or(0);
        let preview_base64 = value
            .get("previewBase64")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        items.push(ProjectSummary {
            id,
            name,
            updated_at,
            image_count,
            preview_base64,
        });
    }

    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(items)
}

pub fn load_image_editor_project(app: &AppHandle, id: String) -> Result<String, String> {
    let path = project_path(app, &id)?;
    if !path.is_file() {
        return Err("项目不存在".into());
    }
    fs::read_to_string(&path).map_err(|e| format!("读取项目失败: {e}"))
}

pub fn delete_image_editor_project(app: &AppHandle, id: String) -> Result<(), String> {
    let path = project_path(app, &id)?;
    if path.is_file() {
        fs::remove_file(&path).map_err(|e| format!("删除项目失败: {e}"))?;
    }
    Ok(())
}
