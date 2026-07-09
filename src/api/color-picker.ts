/**
 * color-picker.ts
 * 取色器前端 IPC 封装层
 */
import { invoke } from '@tauri-apps/api/core'

/** 显示器信息 */
export interface MonitorInfo {
  index: number
  label: string
  x: number
  y: number
  width: number
  height: number
  is_primary: boolean
}

/** 启动/刷新取色时返回的屏幕快照 */
export interface StartPickerResult {
  image_base64: string
  width: number
  height: number
  origin_x: number
  origin_y: number
  radius: number
  monitor_index: number
  monitor_label: string
  capture_all: boolean
}

/** 取色结束结果 */
export interface FinishPickerResult {
  copied: boolean
  hex?: string
}

/** 可选放大倍数：radius → 网格边长 (2×radius+1) */
export const MAGNIFY_OPTIONS = [
  { label: '关闭', radius: 0 },
  { label: '3×', radius: 1 },
  { label: '5×', radius: 2 },
  { label: '7×', radius: 3 },
  { label: '9×', radius: 4 },
  { label: '11×', radius: 5 },
  { label: '13×', radius: 6 },
  { label: '17×', radius: 8 },
] as const

/** 取色窗口启动参数（入口页 → 独立窗口） */
export interface PickerLaunchConfig {
  radius: number
  hideApp: boolean
  monitorIndex: number
  captureAll: boolean
}

/** 枚举可用显示器 */
export const listPickerMonitors = (): Promise<MonitorInfo[]> => {
  return invoke<MonitorInfo[]>('list_picker_monitors')
}

export const preparePickerLaunch = (config: PickerLaunchConfig): Promise<void> =>
  invoke<void>('prepare_picker_launch', { config })

export const takePickerLaunch = (): Promise<PickerLaunchConfig | null> =>
  invoke<PickerLaunchConfig | null>('take_picker_launch')

export const openColorPickerWindow = (): Promise<void> =>
  invoke<void>('open_color_picker_window')

/** 截屏并启动取色会话，返回 PNG 快照 */
export const startPicker = (
  radius = 0,
  hideApp = true,
  monitorIndex = 0,
  captureAll = false,
): Promise<StartPickerResult> => {
  return invoke<StartPickerResult>('start_picker', {
    radius,
    hideApp,
    monitorIndex,
    captureAll,
  })
}

/** 取色会话中重新截屏 */
export const refreshPicker = (
  hideApp = true,
  monitorIndex = 0,
  captureAll = false,
  radius?: number,
): Promise<StartPickerResult> => {
  return invoke<StartPickerResult>('refresh_picker', {
    hideApp,
    monitorIndex,
    captureAll,
    radius,
  })
}

export const finishPicker = (
  cancel: boolean,
  r?: number,
  g?: number,
  b?: number,
): Promise<FinishPickerResult> => {
  return invoke<FinishPickerResult>('finish_picker', { cancel, r, g, b })
}

/** 由 radius 计算网格边长与中心索引 */
export const gridMeta = (radius: number): { gridSize: number; centerIndex: number } => {
  const gridSize = radius > 0 ? radius * 2 + 1 : 1
  return { gridSize, centerIndex: radius > 0 ? radius * gridSize + radius : 0 }
}
