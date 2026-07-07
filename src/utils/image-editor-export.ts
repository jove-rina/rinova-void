/**
 * image-editor-export.ts
 * 图片编辑器 — 导出格式与编码
 */
export interface ImageExportFormat {
  id: string
  label: string
  mime: string
  ext: string
  /** JPEG / WebP 质量，0–1 */
  quality?: number
}

/** 浏览器 Canvas 与 Rust image crate 共同支持的常见格式 */
export const IMAGE_EXPORT_FORMATS: ImageExportFormat[] = [
  { id: 'png', label: 'PNG', mime: 'image/png', ext: 'png' },
  { id: 'jpeg', label: 'JPEG', mime: 'image/jpeg', ext: 'jpg', quality: 0.92 },
  { id: 'webp', label: 'WebP', mime: 'image/webp', ext: 'webp', quality: 0.92 },
  { id: 'bmp', label: 'BMP', mime: 'image/bmp', ext: 'bmp' },
  { id: 'gif', label: 'GIF', mime: 'image/gif', ext: 'gif' },
  { id: 'tiff', label: 'TIFF', mime: 'image/tiff', ext: 'tiff' },
  { id: 'ico', label: 'ICO', mime: 'image/x-icon', ext: 'ico' },
  { id: 'avif', label: 'AVIF', mime: 'image/avif', ext: 'avif', quality: 0.85 },
]

const CANVAS_NATIVE_MIMES = new Set(['image/png', 'image/jpeg', 'image/webp'])

export const formatById = (id: string): ImageExportFormat =>
  IMAGE_EXPORT_FORMATS.find((f) => f.id === id) ?? IMAGE_EXPORT_FORMATS[0]

export const canvasSupportsMime = (mime: string): boolean => CANVAS_NATIVE_MIMES.has(mime)

/** Canvas → Blob（原生 MIME） */
export const canvasToBlob = (
  canvas: HTMLCanvasElement,
  format: ImageExportFormat,
): Promise<Blob> =>
  new Promise((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (blob) resolve(blob)
        else reject(new Error(`无法编码为 ${format.label}`))
      },
      format.mime,
      format.quality,
    )
  })

/** Canvas → RGBA 字节（供 Rust 转码非原生格式） */
export const canvasToRgba = (canvas: HTMLCanvasElement): Uint8ClampedArray => {
  const ctx = canvas.getContext('2d')
  if (!ctx) throw new Error('无法读取画布像素')
  return ctx.getImageData(0, 0, canvas.width, canvas.height).data
}

export const buildExportFilename = (baseName: string, format: ImageExportFormat): string => {
  const stem = baseName.replace(/\.[^.]+$/, '') || 'void-image'
  const stamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
  return `${stem}-${stamp}.${format.ext}`
}
