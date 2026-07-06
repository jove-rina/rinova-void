//! 取色器共享类型：截屏缓冲、会话状态、前后端 DTO。

use std::io::Cursor;
use std::sync::Mutex;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageBuffer, RgbaImage};
use serde::{Deserialize, Serialize};
use tauri::{PhysicalPosition, PhysicalSize};

/// 放大镜最大半径（对应前端最大网格 17×17）。
pub const MAX_RADIUS: u8 = 8;

/// 一次 GDI / CGDisplay 截屏的原始像素与虚拟桌面坐标。
pub(crate) struct ScreenCapture {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: u32,
    pub height: u32,
    /// BGRA 源数据经 swap 后得到的 RGBA 像素（行优先，每像素 4 字节）。
    pub rgba: Vec<u8>,
}

/// 活跃取色会话在 Rust 侧仅保留放大镜半径；图像已以 base64 交给前端 Canvas。
pub(crate) struct PickerSession {
    pub radius: u8,
}

/// 由 Tauri manage 注入的全局取色状态。
pub struct PickerState {
    pub(crate) is_active: Mutex<bool>,
    pub(crate) session: Mutex<Option<PickerSession>>,
    /// macOS：取色开始前保存的窗口几何，结束时恢复（Windows 不使用）。
    pub(crate) saved_layout: Mutex<Option<SavedWindowLayout>>,
}

/// macOS 取色窗口布局前的几何快照。
pub(crate) struct SavedWindowLayout {
    pub position: PhysicalPosition<i32>,
    pub size: PhysicalSize<u32>,
    pub resizable: bool,
    pub fullscreen: bool,
}

impl Default for PickerState {
    fn default() -> Self {
        Self {
            is_active: Mutex::new(false),
            session: Mutex::new(None),
            saved_layout: Mutex::new(None),
        }
    }
}

/// 枚举显示器条目（`list_monitors` 命令返回值元素）。
#[derive(Serialize, Deserialize, Clone)]
pub struct MonitorInfo {
    pub index: u32,
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// `start_picker` / `refresh_picker` 返回给前端的截屏与布局元数据。
#[derive(Serialize, Deserialize, Clone)]
pub struct StartPickerResult {
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub radius: u8,
    pub monitor_index: u32,
    pub monitor_label: String,
    pub capture_all: bool,
}

/// `finish_picker` 返回值：是否已复制 HEX 到剪贴板。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinishPickerResult {
    pub copied: bool,
    pub hex: Option<String>,
}

impl ScreenCapture {
    /// 将 RGBA 缓冲编码为 PNG 并做 standard base64，供前端 `<img>` / Canvas 使用。
    pub fn to_png_base64(&self) -> Result<String, String> {
        let img: RgbaImage = ImageBuffer::from_raw(self.width, self.height, self.rgba.clone())
            .ok_or_else(|| "图像缓冲无效".to_string())?;
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| format!("PNG 编码失败: {e}"))?;
        Ok(STANDARD.encode(buf.into_inner()))
    }
}

/// 将 RGB 分量格式化为 `#rrggbb` 小写 HEX 字符串。
pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
