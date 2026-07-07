use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::{ImageBuffer, ImageFormat, RgbaImage};
use tauri::{AppHandle, Manager};

fn sanitize_filename(filename: &str) -> Result<String, String> {
    Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && !name.contains(".."))
        .map(str::to_string)
        .ok_or_else(|| "文件名无效".to_string())
}

fn downloads_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("无法获取下载目录: {e}"))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建下载目录失败: {e}"))?;
    Ok(dir)
}

pub fn export_text_to_downloads(
    app: &AppHandle,
    filename: &str,
    content: &str,
) -> Result<String, String> {
    let safe_name = sanitize_filename(filename)?;
    let dir = downloads_dir(app)?;
    let path = dir.join(safe_name);
    std::fs::write(&path, content.as_bytes()).map_err(|e| format!("写入文件失败: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

pub fn export_binary_to_downloads(
    app: &AppHandle,
    filename: &str,
    content: &[u8],
) -> Result<String, String> {
    let safe_name = sanitize_filename(filename)?;
    let dir = downloads_dir(app)?;
    let path = dir.join(safe_name);
    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

fn parse_image_format(format: &str) -> Result<ImageFormat, String> {
    match format.to_ascii_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::WebP),
        "gif" => Ok(ImageFormat::Gif),
        "bmp" => Ok(ImageFormat::Bmp),
        "tiff" | "tif" => Ok(ImageFormat::Tiff),
        "ico" => Ok(ImageFormat::Ico),
        "avif" => Ok(ImageFormat::Avif),
        other => Err(format!("不支持的图片格式: {other}")),
    }
}

pub fn export_rgba_image(
    app: &AppHandle,
    filename: &str,
    format: &str,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
) -> Result<String, String> {
    let safe_name = sanitize_filename(filename)?;
    let image_format = parse_image_format(format)?;

    let img: RgbaImage = ImageBuffer::from_raw(width, height, rgba)
        .ok_or_else(|| "图片像素数据无效".to_string())?;

    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image_format)
        .map_err(|e| format!("编码图片失败: {e}"))?;

    export_binary_to_downloads(app, &safe_name, buf.get_ref())
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
