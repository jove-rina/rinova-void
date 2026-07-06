//! macOS 取色窗口布局：铺满 work area 并在结束时恢复。

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition};

use crate::color_picker::macos::{hide_window, run_on_main_thread};
use crate::color_picker::types::{PickerState, SavedWindowLayout};

/// 将主窗口对齐到目标显示器的 work area（可见区域，不含菜单栏 / Dock）。
pub fn layout_picker_window(
    app: &AppHandle,
    _state: &PickerState,
    monitor_index: u32,
    capture_all: bool,
) -> Result<(), String> {
    let app = app.clone();
    run_on_main_thread(app.clone(), move || {
        let state = app.state::<PickerState>();
        layout_picker_window_inner(&app, state.inner(), monitor_index, capture_all)
    })
}

fn layout_picker_window_inner(
    app: &AppHandle,
    state: &PickerState,
    monitor_index: u32,
    capture_all: bool,
) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口未找到".to_string())?;

    {
        let mut saved = state.saved_layout.lock().map_err(|e| e.to_string())?;
        if saved.is_none() {
            *saved = Some(SavedWindowLayout {
                position: main.outer_position().map_err(|e| e.to_string())?,
                size: main.outer_size().map_err(|e| e.to_string())?,
                resizable: main.is_resizable().map_err(|e| e.to_string())?,
                fullscreen: main.is_fullscreen().map_err(|e| e.to_string())?,
            });
        }
    }

    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    if monitors.is_empty() {
        return Err("未检测到可用显示器".into());
    }

    let mut sorted: Vec<_> = monitors.into_iter().collect();
    sorted.sort_by_key(|m| {
        let p = m.position();
        (p.x, p.y)
    });

    let (x, y, logical_w, logical_h) = if capture_all {
        let min_x = sorted
            .iter()
            .map(|m| m.work_area().position.x)
            .min()
            .unwrap();
        let min_y = sorted
            .iter()
            .map(|m| m.work_area().position.y)
            .min()
            .unwrap();
        let max_x = sorted
            .iter()
            .map(|m| m.work_area().position.x + m.work_area().size.width as i32)
            .max()
            .unwrap();
        let max_y = sorted
            .iter()
            .map(|m| m.work_area().position.y + m.work_area().size.height as i32)
            .max()
            .unwrap();
        let scale = sorted.first().map(|m| m.scale_factor()).unwrap_or(1.0);
        let phys_w = (max_x - min_x).max(1) as f64;
        let phys_h = (max_y - min_y).max(1) as f64;
        (
            min_x,
            min_y,
            (phys_w / scale).round(),
            (phys_h / scale).round(),
        )
    } else {
        let monitor = sorted
            .get(monitor_index as usize)
            .ok_or_else(|| format!("显示器 {} 不存在", monitor_index))?;
        let scale = monitor.scale_factor();
        let work = monitor.work_area();
        (
            work.position.x,
            work.position.y,
            (work.size.width as f64 / scale).round(),
            (work.size.height as f64 / scale).round(),
        )
    };

    main.set_fullscreen(false).map_err(|e| e.to_string())?;
    main.set_resizable(true).map_err(|e| e.to_string())?;
    main.set_size(LogicalSize::new(logical_w, logical_h))
        .map_err(|e| e.to_string())?;
    main.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    hide_window::present_window_on_main_thread(&main)
}

/// 结束取色时恢复 `saved_layout` 中保存的窗口几何。
pub fn restore_picker_window(app: &AppHandle, _state: &PickerState) -> Result<(), String> {
    let app = app.clone();
    run_on_main_thread(app.clone(), move || {
        let state = app.state::<PickerState>();
        let Some(main) = app.get_webview_window("main") else {
            return Ok(());
        };

        let layout = state
            .saved_layout
            .lock()
            .map_err(|e| e.to_string())?
            .take();
        if let Some(layout) = layout {
            let _ = main.set_fullscreen(layout.fullscreen);
            let _ = main.set_resizable(layout.resizable);
            let _ = main.set_size(layout.size);
            let _ = main.set_position(layout.position);
            hide_window::present_window_on_main_thread(&main)?;
        }
        Ok(())
    })
}

/// 在截屏 hide 之前保存当前设置页窗口几何（此时窗口仍可见）。
pub fn save_pre_picker_layout(app: &AppHandle) -> Result<(), String> {
    let app = app.clone();
    run_on_main_thread(app.clone(), move || {
        let state = app.state::<PickerState>();
        let main = app
            .get_webview_window("main")
            .ok_or_else(|| "主窗口未找到".to_string())?;
        let mut saved = state.inner().saved_layout.lock().map_err(|e| e.to_string())?;
        if saved.is_none() {
            *saved = Some(SavedWindowLayout {
                position: main.outer_position().map_err(|e| e.to_string())?,
                size: main.outer_size().map_err(|e| e.to_string())?,
                resizable: main.is_resizable().map_err(|e| e.to_string())?,
                fullscreen: main.is_fullscreen().map_err(|e| e.to_string())?,
            });
        }
        Ok(())
    })
}
