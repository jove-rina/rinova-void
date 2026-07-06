//! 非 Windows / 非 macOS 平台的占位实现（统一返回「暂未支持」）。

use crate::color_picker::types::{MonitorInfo, ScreenCapture};

pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    Err("当前平台取色暂未支持".into())
}

pub fn capture_virtual_desktop() -> Result<ScreenCapture, String> {
    Err("当前平台取色暂未支持".into())
}

pub fn capture_monitor(_index: u32) -> Result<ScreenCapture, String> {
    Err("当前平台取色暂未支持".into())
}
