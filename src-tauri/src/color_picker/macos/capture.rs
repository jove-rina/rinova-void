//! macOS 截屏：CGDisplay / 窗口列表合成（可排除本进程窗口）。

use core_foundation::array::{CFArray, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation::base::{CFIndex, TCFType};
use core_foundation::dictionary::{CFDictionaryGetValueIfPresent, CFDictionaryRef};
use core_foundation::number::{kCFNumberSInt32Type, CFNumber, CFNumberGetValue, CFNumberRef};
use core_foundation::string::CFStringRef;
use core_graphics::display::CGDisplay;
use core_graphics::geometry::CGRect;
use core_graphics::image::CGImage;
use core_graphics::window::{
    copy_window_info, create_image, create_image_from_array, kCGNullWindowID,
    kCGWindowImageBestResolution, kCGWindowListOptionOnScreenOnly, kCGWindowNumber,
    kCGWindowOwnerPID,
};
use display_info::DisplayInfo;
use std::os::raw::c_void;
use std::process;
use tauri::AppHandle;

use crate::color_picker::macos::screen_access::{
    capture_api_failed_message, ensure_screen_capture_access, has_screen_capture_access,
    permission_denied_message,
};
use crate::color_picker::types::{MonitorInfo, ScreenCapture};

fn sorted_monitors(app: &AppHandle) -> Result<Vec<tauri::Monitor>, String> {
    let mut monitors = app.available_monitors().map_err(|e| e.to_string())?;
    monitors.sort_by_key(|m| {
        let p = m.position();
        (p.x, p.y)
    });
    Ok(monitors)
}

fn monitor_label(index: usize, width: u32, height: u32, scale: f64, is_primary: bool) -> String {
    let scale_tag = if (scale - 1.0).abs() > 0.01 {
        format!(" · {}%", (scale * 100.0).round() as u32)
    } else {
        String::new()
    };
    let primary_tag = if is_primary { " · 主屏" } else { "" };
    format!(
        "显示器 {} · {}×{}{}{}",
        index + 1,
        width,
        height,
        scale_tag,
        primary_tag
    )
}

fn display_physical_size(display: &DisplayInfo) -> (u32, u32) {
    let w = (display.width as f64 * display.scale_factor as f64).round() as u32;
    let h = (display.height as f64 * display.scale_factor as f64).round() as u32;
    (w, h)
}

/// 按 Tauri 显示器顺序匹配系统 DisplayInfo（物理分辨率对齐）
fn displays_for_monitors(app: &AppHandle) -> Result<Vec<DisplayInfo>, String> {
    let monitors = sorted_monitors(app)?;
    let displays = DisplayInfo::all().map_err(|e| format!("枚举显示器失败: {e}"))?;
    let mut used = vec![false; displays.len()];
    let mut matched = Vec::with_capacity(monitors.len());

    for monitor in &monitors {
        let size = monitor.size();
        let mut pick: Option<usize> = None;

        for (idx, display) in displays.iter().enumerate() {
            if used[idx] {
                continue;
            }
            let (w, h) = display_physical_size(display);
            if w == size.width && h == size.height {
                pick = Some(idx);
                break;
            }
        }

        if pick.is_none() {
            pick = displays
                .iter()
                .enumerate()
                .find(|(i, _)| !used[*i])
                .map(|(i, _)| i);
        }

        let idx = pick.ok_or_else(|| "无法匹配显示器截屏源".to_string())?;
        used[idx] = true;
        matched.push(displays[idx]);
    }

    Ok(matched)
}

fn remove_extra_row_bytes(
    width: usize,
    height: usize,
    bytes_per_row: usize,
    buf: Vec<u8>,
) -> Vec<u8> {
    let row_bytes = width * 4;
    if bytes_per_row <= row_bytes {
        return buf;
    }
    let mut out = Vec::with_capacity(row_bytes * height);
    for row in buf.chunks_exact(bytes_per_row) {
        out.extend_from_slice(&row[..row_bytes]);
    }
    out
}

fn bgra_to_rgba(width: u32, height: u32, buf: Vec<u8>) -> Result<Vec<u8>, String> {
    let expected = (width as usize) * (height as usize) * 4;
    if buf.len() < expected {
        return Err(format!(
            "截屏缓冲不足：期望 {} 字节，实际 {} 字节",
            expected,
            buf.len()
        ));
    }

    let mut rgba = buf;
    for chunk in rgba.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
    Ok(rgba)
}

fn cg_image_to_rgba(cg_image: &CGImage) -> Result<(u32, u32, Vec<u8>), String> {
    let width = cg_image.width() as u32;
    let height = cg_image.height() as u32;
    if width == 0 || height == 0 {
        return Err("截屏结果尺寸无效".into());
    }

    let bytes_per_row = cg_image.bytes_per_row();
    let raw = cg_image.data().bytes().to_vec();
    let clean = remove_extra_row_bytes(width as usize, height as usize, bytes_per_row, raw);
    let rgba = bgra_to_rgba(width, height, clean)?;
    Ok((width, height, rgba))
}

unsafe fn dict_get_i32(dict: CFDictionaryRef, key: CFStringRef) -> Option<i32> {
    let mut value: *const c_void = std::ptr::null();
    if CFDictionaryGetValueIfPresent(dict, key as *const c_void, &mut value) == 0 {
        return None;
    }
    let mut out: i32 = 0;
    if CFNumberGetValue(
        value as CFNumberRef,
        kCFNumberSInt32Type,
        &mut out as *mut i32 as *mut c_void,
    ) {
        Some(out)
    } else {
        None
    }
}

/// 收集当前屏幕上除本进程外的窗口 ID，供 `CGWindowListCreateImageFromArray` 合成。
fn window_ids_excluding_own_pid() -> Result<CFArray, String> {
    let info = copy_window_info(kCGWindowListOptionOnScreenOnly, kCGNullWindowID)
        .ok_or_else(|| "无法枚举窗口".to_string())?;
    let own_pid = process::id() as i32;
    let mut ids: Vec<CFNumber> = Vec::new();

    for i in 0..info.len() {
        unsafe {
            let dict = CFArrayGetValueAtIndex(
                info.as_CFTypeRef() as CFArrayRef,
                i as CFIndex,
            ) as CFDictionaryRef;

            let owner_pid = dict_get_i32(dict, kCGWindowOwnerPID).unwrap_or(-1);
            if owner_pid == own_pid {
                continue;
            }

            let Some(window_id) = dict_get_i32(dict, kCGWindowNumber) else {
                continue;
            };
            if window_id <= 0 {
                continue;
            }
            ids.push(CFNumber::from(window_id));
        }
    }

    if ids.is_empty() {
        return Err("截屏失败：排除本应用后无可用窗口层（请确认桌面可见）".into());
    }

    Ok(CFArray::<CFNumber>::from_CFTypes(&ids).into_untyped())
}

/// 在指定矩形内合成屏幕内容；`exclude_own_windows` 时跳过本进程全部窗口。
fn capture_region(bounds: CGRect, exclude_own_windows: bool) -> Result<(u32, u32, Vec<u8>), String> {
    ensure_screen_capture_access()?;
    let cg_image = if exclude_own_windows {
        let window_ids = window_ids_excluding_own_pid()?;
        create_image_from_array(bounds, window_ids, kCGWindowImageBestResolution)
    } else {
        create_image(
            bounds,
            kCGWindowListOptionOnScreenOnly,
            kCGNullWindowID,
            kCGWindowImageBestResolution,
        )
    }
    .ok_or_else(capture_api_failed_message)?;
    cg_image_to_rgba(&cg_image)
}

fn capture_display(display_id: u32, exclude_own_windows: bool) -> Result<(u32, u32, Vec<u8>), String> {
    ensure_screen_capture_access()?;
    let display = CGDisplay::new(display_id);

    if exclude_own_windows {
        let bounds = display.bounds();
        // 优先窗口列表合成（可排除本进程窗口）；release 上该 API 偶发返回空，回退 CGDisplay。
        if let Ok(result) = capture_region(bounds, true) {
            return Ok(result);
        }
        // 截屏前已 orderOut + hide，CGDisplay 通常不会包含本应用。
        let cg_image = display.image().ok_or_else(capture_api_failed_message)?;
        return cg_image_to_rgba(&cg_image);
    }

    let cg_image = display.image().ok_or_else(capture_api_failed_message)?;
    cg_image_to_rgba(&cg_image)
}

fn validate_capture_size(
    expected_w: u32,
    expected_h: u32,
    actual_w: u32,
    actual_h: u32,
) -> Result<(), String> {
    let min_w = (expected_w as f64 * 0.45).round() as u32;
    let min_h = (expected_h as f64 * 0.45).round() as u32;
    if actual_w < min_w || actual_h < min_h {
        if !has_screen_capture_access() {
            return Err(permission_denied_message());
        }
        return Err(format!(
            "截屏失败（{}×{}），请重试或关闭「截屏时隐藏应用」。",
            actual_w, actual_h
        ));
    }
    Ok(())
}

fn image_to_capture(
    origin_x: i32,
    origin_y: i32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
) -> ScreenCapture {
    ScreenCapture {
        origin_x,
        origin_y,
        width,
        height,
        rgba,
    }
}

fn blit_rgba(
    dest: &mut [u8],
    dest_w: u32,
    dest_h: u32,
    dest_x: u32,
    dest_y: u32,
    src_w: u32,
    src_h: u32,
    src: &[u8],
) {
    for row in 0..src_h {
        if dest_y + row >= dest_h {
            break;
        }
        for col in 0..src_w {
            if dest_x + col >= dest_w {
                break;
            }
            let src_idx = ((row * src_w + col) * 4) as usize;
            let dst_idx = (((dest_y + row) * dest_w + dest_x + col) * 4) as usize;
            if src_idx + 3 < src.len() && dst_idx + 3 < dest.len() {
                dest[dst_idx..dst_idx + 4].copy_from_slice(&src[src_idx..src_idx + 4]);
            }
        }
    }
}

pub fn list_monitors(app: &AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let monitors = sorted_monitors(app)?;
    let displays = displays_for_monitors(app).unwrap_or_default();

    Ok(monitors
        .iter()
        .enumerate()
        .map(|(idx, monitor)| {
            let p = monitor.position();
            let s = monitor.size();
            let scale = monitor.scale_factor();
            let is_primary = displays
                .get(idx)
                .map(|d| d.is_primary)
                .unwrap_or(idx == 0);
            MonitorInfo {
                index: idx as u32,
                label: monitor_label(idx, s.width, s.height, scale, is_primary),
                x: p.x,
                y: p.y,
                width: s.width,
                height: s.height,
                is_primary,
            }
        })
        .collect())
}

pub fn capture_monitor(
    app: &AppHandle,
    index: u32,
    exclude_own_windows: bool,
) -> Result<ScreenCapture, String> {
    let monitors = sorted_monitors(app)?;
    let displays = displays_for_monitors(app)?;
    let monitor = monitors
        .get(index as usize)
        .ok_or_else(|| format!("显示器 {} 不存在", index))?;
    let display = displays
        .get(index as usize)
        .ok_or_else(|| format!("显示器 {} 截屏源不存在", index))?;
    let p = monitor.position();
    let s = monitor.size();
    let (width, height, rgba) = capture_display(display.id, exclude_own_windows)?;
    validate_capture_size(s.width, s.height, width, height)?;
    Ok(image_to_capture(p.x, p.y, width, height, rgba))
}

pub fn capture_virtual_desktop(
    app: &AppHandle,
    exclude_own_windows: bool,
) -> Result<ScreenCapture, String> {
    let monitors = sorted_monitors(app)?;
    let displays = displays_for_monitors(app)?;
    if monitors.is_empty() {
        return Err("未检测到可用显示器".into());
    }

    let min_x = monitors.iter().map(|m| m.position().x).min().unwrap();
    let min_y = monitors.iter().map(|m| m.position().y).min().unwrap();

    let mut captures: Vec<(u32, u32, u32, u32, Vec<u8>)> = Vec::new();
    for (monitor, display) in monitors.iter().zip(displays.iter()) {
        let s = monitor.size();
        let (width, height, rgba) = capture_display(display.id, exclude_own_windows)?;
        validate_capture_size(s.width, s.height, width, height)?;
        let p = monitor.position();
        captures.push((
            (p.x - min_x) as u32,
            (p.y - min_y) as u32,
            width,
            height,
            rgba,
        ));
    }

    let total_w = captures
        .iter()
        .map(|(x, _, w, _, _)| x + w)
        .max()
        .unwrap_or(0);
    let total_h = captures
        .iter()
        .map(|(_, y, _, h, _)| y + h)
        .max()
        .unwrap_or(0);

    let mut rgba = vec![0u8; (total_w as usize) * (total_h as usize) * 4];
    for (ox, oy, w, h, src) in captures {
        blit_rgba(&mut rgba, total_w, total_h, ox, oy, w, h, &src);
    }

    Ok(ScreenCapture {
        origin_x: min_x,
        origin_y: min_y,
        width: total_w,
        height: total_h,
        rgba,
    })
}
