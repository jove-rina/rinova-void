/**
 * platform.ts
 * 轻量平台检测（Tauri WebView 内可用）
 */

/** 当前是否为 macOS 桌面环境 */
export const isMacOs = (): boolean => {
  if (typeof navigator === 'undefined') return false
  return /Mac/i.test(navigator.platform) || /Mac OS X/i.test(navigator.userAgent)
}
