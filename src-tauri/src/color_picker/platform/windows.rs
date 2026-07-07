//! Windows GDI 截屏：EnumDisplayMonitors + BitBlt + GetDIBits。

use std::cell::RefCell;

use windows::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC,
    GetDIBits, GetMonitorInfoW, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW, SRCCOPY,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};

use crate::color_picker::types::{MonitorInfo, ScreenCapture};

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

/// 枚举所有显示器，填充 `MonitorInfo`（含 DPI 缩放标签与主屏标记）。
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

/// 截取虚拟桌面（多屏 union，坐标系与 `SM_*VIRTUALSCREEN` 一致）。
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

/// 按 `EnumDisplayMonitors` 顺序截取指定 index 的显示器。
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
