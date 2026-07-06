/**
 * export.ts
 * 导出文件到系统下载目录并在文件管理器中定位
 */
import { invoke } from '@tauri-apps/api/core'

export const exportTextToDownloads = (
  filename: string,
  content: string,
): Promise<string> => {
  return invoke<string>('export_text_file', { filename, content })
}

export const revealExportPath = (path: string): Promise<void> => {
  return invoke<void>('reveal_export_path', { path })
}
