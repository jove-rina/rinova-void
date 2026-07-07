/**
 * image-editor.ts
 * 图片编辑器 — Tauri 导出 API
 */
import { invoke } from '@tauri-apps/api/core'
import { revealExportPath } from '@/api/export'
import {
  buildExportFilename,
  canvasSupportsMime,
  canvasToBlob,
  canvasToRgba,
  formatById,
  type ImageExportFormat,
} from '@/utils/image-editor-export'

export const exportBinaryToDownloads = (
  filename: string,
  content: Uint8Array,
): Promise<string> =>
  invoke<string>('export_binary_file', {
    filename,
    content: Array.from(content),
  })

export const exportRgbaImage = (
  filename: string,
  format: string,
  width: number,
  height: number,
  rgba: Uint8ClampedArray,
): Promise<string> =>
  invoke<string>('export_rgba_image', {
    filename,
    format,
    width,
    height,
    rgba: Array.from(rgba),
  })

export const exportCanvasImage = async (
  canvas: HTMLCanvasElement,
  baseName: string,
  formatId: string,
): Promise<string> => {
  const format = formatById(formatId)
  const filename = buildExportFilename(baseName, format)

  if (canvasSupportsMime(format.mime)) {
    const blob = await canvasToBlob(canvas, format)
    const bytes = new Uint8Array(await blob.arrayBuffer())
    return exportBinaryToDownloads(filename, bytes)
  }

  const rgba = canvasToRgba(canvas)
  return exportRgbaImage(filename, format.id, canvas.width, canvas.height, rgba)
}

export { revealExportPath, type ImageExportFormat }
