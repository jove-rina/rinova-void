use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

pub fn export_text_to_downloads(
    app: &AppHandle,
    filename: &str,
    content: &str,
) -> Result<String, String> {
    let safe_name = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && !name.contains(".."))
        .ok_or_else(|| "文件名无效".to_string())?;

    let dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("无法获取下载目录: {e}"))?;

    std::fs::create_dir_all(&dir).map_err(|e| format!("创建下载目录失败: {e}"))?;

    let path = dir.join(safe_name);
    std::fs::write(&path, content.as_bytes()).map_err(|e| format!("写入文件失败: {e}"))?;

    Ok(path.to_string_lossy().to_string())
}

pub fn reveal_export_path(path: &str) -> Result<(), String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("文件不存在".into());
    }
    reveal_in_file_manager(&path)
}

#[cfg(windows)]
fn reveal_in_file_manager(path: &Path) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn()
        .map_err(|e| format!("打开目录失败: {e}"))?;
    Ok(())
}

#[cfg(not(windows))]
fn reveal_in_file_manager(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开目录失败: {e}"))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let dir = path
            .parent()
            .ok_or_else(|| "路径无效".to_string())?
            .to_string_lossy()
            .to_string();

        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("打开目录失败: {e}"))?;
        Ok(())
    }
}
