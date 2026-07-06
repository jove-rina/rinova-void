use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageBuffer, RgbaImage};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

const MAX_RADIUS: u8 = 8;
const CAPTURE_HIDE_MS: u64 = 120;

struct ScreenCapture {
    origin_x: i32,
    origin_y: i32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

struct PickerSession {
    _capture: Arc<ScreenCapture>,
    radius: u8,
}

pub struct PickerState {
    is_active: Mutex<bool>,
    session: Mutex<Option<PickerSession>>,
}

impl Default for PickerState {
    fn default() -> Self {
        Self {
            is_active: Mutex::new(false),
            session: Mutex::new(None),
        }
    }
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinishPickerResult {
    pub copied: bool,
    pub hex: Option<String>,
}

impl ScreenCapture {
    fn to_png_base64(&self) -> Result<String, String> {
        let img: RgbaImage = ImageBuffer::from_raw(self.width, self.height, self.rgba.clone())
            .ok_or_else(|| "图像缓冲无效".to_string())?;
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)
            .map_err(|e| format!("PNG 编码失败: {e}"))?;
        Ok(STANDARD.encode(buf.into_inner()))
    }
}

pub fn setup<M: Manager<tauri::Wry>>(app: &M) -> Result<(), String> {
    app.manage(PickerState::default());
    Ok(())
}

pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    Clipboard::new()
        .map_err(|e| format!("剪贴板不可用: {e}"))?
        .set_text(text)
        .map_err(|e| format!("复制失败: {e}"))
}

pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    platform::list_monitors()
}

struct CaptureSnapshot {
    capture: ScreenCapture,
    monitor_index: u32,
    monitor_label: String,
    capture_all: bool,
}

fn capture_snapshot(
    app: &AppHandle,
    hide_app: bool,
    monitor_index: u32,
    capture_all: bool,
) -> Result<CaptureSnapshot, String> {
    let monitors = list_monitors()?;
    if monitors.is_empty() {
        return Err("未检测到可用显示器".into());
    }

    if !capture_all && monitor_index as usize >= monitors.len() {
        return Err(format!("显示器 {} 不存在", monitor_index));
    }

    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口未找到".to_string())?;

    let was_visible = main.is_visible().unwrap_or(true);
    if hide_app && was_visible {
        main.hide()
            .map_err(|e| format!("隐藏主窗口失败: {e}"))?;
        thread::sleep(Duration::from_millis(CAPTURE_HIDE_MS));
    }

    let snapshot = (|| -> Result<CaptureSnapshot, String> {
        if capture_all {
            let capture = platform::capture_virtual_desktop()?;
            let label = format!(
                "全部屏幕 · {}×{}",
                capture.width, capture.height
            );
            Ok(CaptureSnapshot {
                capture,
                monitor_index: u32::MAX,
                monitor_label: label,
                capture_all: true,
            })
        } else {
            let monitor = &monitors[monitor_index as usize];
            let capture = platform::capture_monitor(monitor.index)?;
            Ok(CaptureSnapshot {
                capture,
                monitor_index: monitor.index,
                monitor_label: monitor.label.clone(),
                capture_all: false,
            })
        }
    })();

    if hide_app && was_visible {
        let _ = main.show();
    }

    snapshot
}

fn build_picker_result(snapshot: &CaptureSnapshot, radius: u8) -> Result<StartPickerResult, String> {
    Ok(StartPickerResult {
        image_base64: snapshot.capture.to_png_base64()?,
        width: snapshot.capture.width,
        height: snapshot.capture.height,
        origin_x: snapshot.capture.origin_x,
        origin_y: snapshot.capture.origin_y,
        radius,
        monitor_index: snapshot.monitor_index,
        monitor_label: snapshot.monitor_label.clone(),
        capture_all: snapshot.capture_all,
    })
}

pub fn start_picker(
    app: AppHandle,
    state: State<PickerState>,
    radius: Option<u8>,
    hide_app: Option<bool>,
    monitor_index: Option<u32>,
    capture_all: Option<bool>,
) -> Result<StartPickerResult, String> {
    let radius_value = radius.unwrap_or(0).min(MAX_RADIUS);
    let hide_app = hide_app.unwrap_or(true);
    let capture_all = capture_all.unwrap_or(false);
    let monitor_index = monitor_index.unwrap_or(0);

    if *state.is_active.lock().map_err(|e| e.to_string())? {
        return Err("取色已在进行中".into());
    }

    let snapshot = capture_snapshot(&app, hide_app, monitor_index, capture_all)?;
    let result = build_picker_result(&snapshot, radius_value)?;

    {
        let mut active = state.is_active.lock().map_err(|e| e.to_string())?;
        *active = true;
    }

    *state
        .session
        .lock()
        .map_err(|e| e.to_string())? = Some(PickerSession {
        _capture: Arc::new(snapshot.capture),
        radius: radius_value,
    });

    Ok(result)
}

pub fn refresh_picker(
    app: AppHandle,
    state: State<PickerState>,
    hide_app: Option<bool>,
    monitor_index: Option<u32>,
    capture_all: Option<bool>,
    radius: Option<u8>,
) -> Result<StartPickerResult, String> {
    let hide_app = hide_app.unwrap_or(true);
    let capture_all = capture_all.unwrap_or(false);
    let monitor_index = monitor_index.unwrap_or(0);

    let session_radius = {
        let active = *state.is_active.lock().map_err(|e| e.to_string())?;
        if !active {
            return Err("取色未开始".into());
        }
        state
            .session
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .map(|session| session.radius)
            .ok_or_else(|| "取色会话无效".to_string())?
    };

    let radius_value = radius.unwrap_or(session_radius).min(MAX_RADIUS);
    let snapshot = capture_snapshot(&app, hide_app, monitor_index, capture_all)?;
    let result = build_picker_result(&snapshot, radius_value)?;

    *state
        .session
        .lock()
        .map_err(|e| e.to_string())? = Some(PickerSession {
        _capture: Arc::new(snapshot.capture),
        radius: radius_value,
    });

    Ok(result)
}

pub fn finish_picker(
    app: AppHandle,
    state: State<PickerState>,
    cancel: bool,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
) -> Result<FinishPickerResult, String> {
    finish_picker_inner(&app, &state, cancel, r, g, b)
}

pub fn cancel_picker(app: &AppHandle) -> Result<(), String> {
    let state: State<PickerState> = app.state();
    let active = *state.is_active.lock().map_err(|e| e.to_string())?;
    if active {
        finish_picker_inner(app, &state, true, None, None, None)?;
    }
    Ok(())
}

pub fn is_picker_active(app: &AppHandle) -> bool {
    let Some(state) = app.try_state::<PickerState>() else {
        return false;
    };
    state
        .is_active
        .lock()
        .map(|active| *active)
        .unwrap_or(false)
}

fn finish_picker_state(
    state: &PickerState,
    cancel: bool,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
) -> Result<FinishPickerResult, String> {
    let mut active = state.is_active.lock().map_err(|e| e.to_string())?;
    if !*active {
        return Ok(FinishPickerResult {
            copied: false,
            hex: None,
        });
    }

    if !cancel && (r.is_none() || g.is_none() || b.is_none()) {
        return Err("缺少颜色分量".into());
    }

    *state.session.lock().map_err(|e| e.to_string())? = None;

    let mut copied = false;
    let mut hex = None;

    if !cancel {
        if let (Some(r), Some(g), Some(b)) = (r, g, b) {
            let h = rgb_to_hex(r, g, b);
            copy_to_clipboard(&h)?;
            copied = true;
            hex = Some(h);
        }
    }

    *active = false;
    Ok(FinishPickerResult { copied, hex })
}

fn finish_picker_inner(
    app: &AppHandle,
    state: &PickerState,
    cancel: bool,
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
) -> Result<FinishPickerResult, String> {
    let result = finish_picker_state(state, cancel, r, g, b)?;

    if let Some(main) = app.get_webview_window("main") {
        let _ = main.set_fullscreen(false);
        let _ = main.set_focus();
    }

    Ok(result)
}

#[cfg(windows)]
mod platform {
    use std::cell::RefCell;

    use windows::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC,
        GetDIBits, GetMonitorInfoW, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW, SRCCOPY,
    };
    use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

    use super::{MonitorInfo, ScreenCapture};

    const MONITORINFOF_PRIMARY: u32 = 0x00000001;

    thread_local! {
        static ENUM_MONITORS: RefCell<Vec<MonitorInfo>> = RefCell::new(Vec::new());
        static ENUM_HANDLES: RefCell<Vec<HMONITOR>> = RefCell::new(Vec::new());
    }

    fn ensure_dpi_awareness() {
        use windows::Win32::UI::HiDpi::{
            SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };

        unsafe {
            let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
    }

    fn monitor_dpi_scale(hmonitor: HMONITOR) -> f32 {
        unsafe {
            let mut dpi_x = 96u32;
            let mut dpi_y = 96u32;
            if GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y).is_ok() {
                return dpi_x as f32 / 96.0;
            }
        }
        1.0
    }

    pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
        unsafe {
            ENUM_MONITORS.with(|cell| {
                ENUM_HANDLES.with(|handles| {
                    cell.borrow_mut().clear();
                    handles.borrow_mut().clear();
                    let _ = EnumDisplayMonitors(
                        HDC::default(),
                        None,
                        Some(enum_monitor_proc),
                        LPARAM(0),
                    );
                    Ok(cell.borrow().clone())
                })
            })
        }
    }

    unsafe extern "system" fn enum_monitor_proc(
        hmonitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        _: LPARAM,
    ) -> BOOL {
        ENUM_MONITORS.with(|cell| {
            ENUM_HANDLES.with(|handles| {
            let mut monitors = cell.borrow_mut();
            handles.borrow_mut().push(hmonitor);
            let mut info = MONITORINFOEXW {
                monitorInfo: MONITORINFO {
                    cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                    ..Default::default()
                },
                ..Default::default()
            };

            if !GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _).as_bool() {
                return TRUE;
            }

            let rect = info.monitorInfo.rcMonitor;
            let width = (rect.right - rect.left).max(0) as u32;
            let height = (rect.bottom - rect.top).max(0) as u32;
            if width == 0 || height == 0 {
                return TRUE;
            }

            let scale = monitor_dpi_scale(hmonitor);
            let scale_tag = if (scale - 1.0).abs() > 0.01 {
                format!(" · {}%", (scale * 100.0).round() as u32)
            } else {
                String::new()
            };
            let is_primary = (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0;
            let index = monitors.len() as u32;
            let primary_tag = if is_primary { " · 主屏" } else { "" };
            let label = format!(
                "显示器 {} · {}×{}{}{}",
                index + 1,
                width,
                height,
                scale_tag,
                primary_tag
            );

            monitors.push(MonitorInfo {
                index,
                label,
                x: rect.left,
                y: rect.top,
                width,
                height,
                is_primary,
            });
            TRUE
            })
        })
    }

    pub fn capture_virtual_desktop() -> Result<ScreenCapture, String> {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
            SM_YVIRTUALSCREEN,
        };

        unsafe {
            let origin_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let origin_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(0) as u32;
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(0) as u32;
            capture_region_impl(origin_x, origin_y, width, height)
        }
    }

    pub fn capture_monitor(index: u32) -> Result<ScreenCapture, String> {
        unsafe {
            ENUM_MONITORS.with(|cell| {
                ENUM_HANDLES.with(|handles| {
                    cell.borrow_mut().clear();
                    handles.borrow_mut().clear();
                    let _ = EnumDisplayMonitors(
                        HDC::default(),
                        None,
                        Some(enum_monitor_proc),
                        LPARAM(0),
                    );

                    let monitor = cell
                        .borrow()
                        .get(index as usize)
                        .ok_or_else(|| format!("显示器 {} 不存在", index))?
                        .clone();

                    capture_region_impl(monitor.x, monitor.y, monitor.width, monitor.height)
                })
            })
        }
    }

    fn capture_region_impl(
        origin_x: i32,
        origin_y: i32,
        width: u32,
        height: u32,
    ) -> Result<ScreenCapture, String> {
        if width == 0 || height == 0 {
            return Err("显示器尺寸无效".into());
        }

        ensure_dpi_awareness();

        unsafe {
            let screen_dc = GetDC(None);
            if screen_dc.0.is_null() {
                return Err("无法获取屏幕设备上下文".into());
            }

            let mem_dc = CreateCompatibleDC(screen_dc);
            if mem_dc.0.is_null() {
                let _ = ReleaseDC(None, screen_dc);
                return Err("无法创建兼容 DC".into());
            }

            let bitmap = CreateCompatibleBitmap(screen_dc, width as i32, height as i32);
            if bitmap.0.is_null() {
                let _ = DeleteDC(mem_dc);
                let _ = ReleaseDC(None, screen_dc);
                return Err("无法创建位图".into());
            }

            let old = SelectObject(mem_dc, bitmap);
            let blt_ok = BitBlt(
                mem_dc,
                0,
                0,
                width as i32,
                height as i32,
                screen_dc,
                origin_x,
                origin_y,
                SRCCOPY,
            )
            .is_ok();

            if !blt_ok {
                SelectObject(mem_dc, old);
                let _ = DeleteObject(bitmap);
                let _ = DeleteDC(mem_dc);
                let _ = ReleaseDC(None, screen_dc);
                return Err("屏幕截屏失败".into());
            }

            let mut info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0 as u32,
                    ..Default::default()
                },
                ..Default::default()
            };

            let buf_len = (width as usize) * (height as usize) * 4;
            let mut rgba = vec![0u8; buf_len];
            let lines = GetDIBits(
                mem_dc,
                bitmap,
                0,
                height,
                Some(rgba.as_mut_ptr() as *mut _),
                &mut info,
                DIB_RGB_COLORS,
            );

            SelectObject(mem_dc, old);
            let _ = DeleteObject(bitmap);
            let _ = DeleteDC(mem_dc);
            let _ = ReleaseDC(None, screen_dc);

            if lines == 0 {
                return Err("读取屏幕像素失败".into());
            }

            for chunk in rgba.chunks_exact_mut(4) {
                chunk.swap(0, 2);
            }

            Ok(ScreenCapture {
                origin_x,
                origin_y,
                width,
                height,
                rgba,
            })
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{MonitorInfo, ScreenCapture};

    pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
        Err("当前平台取色暂未支持".into())
    }

    pub fn capture_virtual_desktop() -> Result<ScreenCapture, String> {
        Err("当前平台取色暂未支持".into())
    }

    pub fn capture_monitor(_index: u32) -> Result<ScreenCapture, String> {
        Err("当前平台取色暂未支持".into())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    fn sample_capture() -> ScreenCapture {
        ScreenCapture {
            origin_x: 0,
            origin_y: 0,
            width: 1,
            height: 1,
            rgba: vec![255, 0, 0, 255],
        }
    }

    fn active_state() -> PickerState {
        PickerState {
            is_active: Mutex::new(true),
            session: Mutex::new(Some(PickerSession {
                _capture: Arc::new(sample_capture()),
                radius: 0,
            })),
        }
    }

    #[test]
    fn rgb_to_hex_formats() {
        assert_eq!(rgb_to_hex(192, 132, 252), "#c084fc");
        assert_eq!(rgb_to_hex(0, 0, 0), "#000000");
        assert_eq!(rgb_to_hex(255, 255, 255), "#ffffff");
    }

    #[test]
    fn finish_inactive_returns_not_copied() {
        let state = PickerState::default();
        let result = finish_picker_state(&state, true, None, None, None).unwrap();
        assert!(!result.copied);
        assert!(result.hex.is_none());
    }

    #[test]
    fn finish_cancel_clears_session_without_copy() {
        let state = active_state();
        let result = finish_picker_state(&state, true, None, None, None).unwrap();
        assert!(!result.copied);
        assert!(result.hex.is_none());
        assert!(!*state.is_active.lock().unwrap());
        assert!(state.session.lock().unwrap().is_none());
    }

    #[test]
    fn finish_confirm_requires_rgb_components() {
        let state = active_state();
        let err = finish_picker_state(&state, false, None, None, None).unwrap_err();
        assert_eq!(err, "缺少颜色分量");
        assert!(*state.is_active.lock().unwrap());
        assert!(state.session.lock().unwrap().is_some());
    }
}
