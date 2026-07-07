use std::io::Cursor;
use std::path::{Path, PathBuf};

use base64::Engine;
use image::{ImageBuffer, ImageFormat, RgbaImage};
use tauri::{AppHandle, Manager};

fn decode_base64_payload(content_base64: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(content_base64.trim())
        .map_err(|e| format!("解码图片数据失败: {e}"))
}

fn sanitize_filename(filename: &str) -> Result<String, String> {
    Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && !name.contains(".."))
        .map(str::to_string)
        .ok_or_else(|| "文件名无效".to_string())
}

fn validate_dest_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        return Err("保存路径无效".into());
    }

    let valid_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && !name.contains(".."));

    if valid_name.is_none() {
        return Err("文件名无效".into());
    }

    Ok(path)
}

pub fn write_binary_to_path(path: &str, content: &[u8]) -> Result<String, String> {
    let path = validate_dest_path(path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

pub fn export_binary_base64_to_path(path: &str, content_base64: &str) -> Result<String, String> {
    let bytes = decode_base64_payload(content_base64)?;
    write_binary_to_path(path, &bytes)
}

pub fn convert_image_bytes_to_path(path: &str, format: &str, bytes: &[u8]) -> Result<String, String> {
    let image_format = parse_image_format(format)?;
    let img = image::load_from_memory(bytes).map_err(|e| format!("无法解析图片: {e}"))?;

    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image_format)
        .map_err(|e| format!("编码图片失败: {e}"))?;

    write_binary_to_path(path, buf.get_ref())
}

pub fn convert_image_base64_to_path(
    path: &str,
    format: &str,
    content_base64: &str,
) -> Result<String, String> {
    let bytes = decode_base64_payload(content_base64)?;
    convert_image_bytes_to_path(path, format, &bytes)
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

pub fn export_binary_base64_to_downloads(
    app: &AppHandle,
    filename: &str,
    content_base64: &str,
) -> Result<String, String> {
    let bytes = decode_base64_payload(content_base64)?;
    export_binary_to_downloads(app, filename, &bytes)
}

pub fn parse_image_format(format: &str) -> Result<ImageFormat, String> {
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

/// 将已编码图片（如 PNG base64）转码为其他格式后写入下载目录。
pub fn convert_image_base64(
    app: &AppHandle,
    filename: &str,
    format: &str,
    content_base64: &str,
) -> Result<String, String> {
    let safe_name = sanitize_filename(filename)?;
    let image_format = parse_image_format(format)?;
    let bytes = decode_base64_payload(content_base64)?;

    let img = image::load_from_memory(&bytes).map_err(|e| format!("无法解析图片: {e}"))?;

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
