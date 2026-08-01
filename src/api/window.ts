/**
 * window.ts
 * 主窗口相关 IPC
 */
import { invoke } from '@tauri-apps/api/core'

/** 将主窗口恢复为默认尺寸并居中显示 */
export const resetMainWindow = (): Promise<void> =>
  invoke<void>('reset_window')
