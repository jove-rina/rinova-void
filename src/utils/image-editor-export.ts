/**
 * image-editor-export.ts
 * 图片编辑器 — 导出格式与编码
 */
export interface ImageCompressionPreset {
  id: string
  label: string
  /** undefined 表示使用格式默认质量（100%） */
  quality?: number
}

export interface ImageSizeScalePreset {
  id: string
  label: string
  /** 相对当前画布的缩放比，1 = 100% */
  scale: number
}

export interface ExportSizeOptions {
  width: number
  height: number
}

export type ExportMode = 'general' | 'thumbnail' | 'ico'

export interface ExportSettings {
  mode: ExportMode
  exportName: string
  formatId: string
  compressionPresetId: string
  exportSize: ExportSizeOptions
  thumbnailSizes: number[]
  icoSizes: number[]
}

/** 缩略图可选正方形尺寸 */
export const THUMBNAIL_SIZE_OPTIONS = [16, 32, 48, 64, 128, 256, 512, 1024] as const

/** ICO 格式最大边长（image crate 限制为 256） */
export const ICO_MAX_SIZE = 256

/** ICO 可选尺寸 */
export const ICO_SIZE_OPTIONS = THUMBNAIL_SIZE_OPTIONS.filter(
  (size) => size <= ICO_MAX_SIZE,
)

export type ThumbnailSize = (typeof THUMBNAIL_SIZE_OPTIONS)[number]
export type IcoSize = (typeof ICO_SIZE_OPTIONS)[number]

export const isValidIcoSize = (size: number): boolean =>
  size >= 1 && size <= ICO_MAX_SIZE && ICO_SIZE_OPTIONS.includes(size as IcoSize)

const EXPORT_PERCENT_LEVELS = [100, 80, 60, 50, 30, 20] as const

/** 导出压缩档位（编码质量） */
export const IMAGE_COMPRESSION_PRESETS: ImageCompressionPreset[] = EXPORT_PERCENT_LEVELS.map(
  (percent) => ({
    id: String(percent),
    label: `${percent}%`,
    quality: percent === 100 ? undefined : percent / 100,
  }),
)

/** 导出尺寸缩放档位 */
export const IMAGE_SIZE_SCALE_PRESETS: ImageSizeScalePreset[] = EXPORT_PERCENT_LEVELS.map(
  (percent) => ({
    id: String(percent),
    label: `${percent}%`,
    scale: percent / 100,
  }),
)

export const compressionById = (id: string): ImageCompressionPreset =>
  IMAGE_COMPRESSION_PRESETS.find((p) => p.id === id) ?? IMAGE_COMPRESSION_PRESETS[0]

export const sizeScaleById = (id: string): ImageSizeScalePreset =>
  IMAGE_SIZE_SCALE_PRESETS.find((p) => p.id === id) ?? IMAGE_SIZE_SCALE_PRESETS[0]

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

/** 单次 IPC 可安全传输的最大字节数，超出则走分块导出 */
export const EXPORT_SINGLE_SHOT_MAX_BYTES = 512 * 1024

/** 分块导出时每块原始字节数 */
export const EXPORT_CHUNK_BYTES = 256 * 1024

/** Canvas → Blob（比 toDataURL 更省内存） */
export const canvasToBlob = (
  canvas: HTMLCanvasElement,
  format: ImageExportFormat,
): Promise<Blob> =>
  new Promise((resolve, reject) => {
    const onBlob = (blob: Blob | null) => {
      if (!blob || blob.size === 0) {
        reject(new Error(`无法编码为 ${format.label}`))
        return
      }
      resolve(blob)
    }

    if (format.quality != null && format.mime !== 'image/png') {
      canvas.toBlob(onBlob, format.mime, format.quality)
    } else {
      canvas.toBlob(onBlob, format.mime)
    }
  })

/** Canvas → 原始字节 */
export const canvasToBytes = async (
  canvas: HTMLCanvasElement,
  format: ImageExportFormat,
): Promise<Uint8Array> => {
  try {
    const blob = await canvasToBlob(canvas, format)
    return new Uint8Array(await blob.arrayBuffer())
  } catch {
    const base64 = canvasToBase64(canvas, format)
    return base64ToBytes(base64)
  }
}

export const base64ToBytes = (base64: string): Uint8Array => {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}

export const uint8ToBase64 = (bytes: Uint8Array): string => {
  let binary = ''
  const step = 0x8000
  for (let i = 0; i < bytes.length; i += step) {
    binary += String.fromCharCode(...bytes.subarray(i, i + step))
  }
  return btoa(binary)
}

export const bytesToBase64Chunks = (
  data: Uint8Array,
  chunkBytes = EXPORT_CHUNK_BYTES,
): string[] => {
  const chunks: string[] = []
  for (let i = 0; i < data.length; i += chunkBytes) {
    const slice = data.subarray(i, Math.min(i + chunkBytes, data.length))
    chunks.push(uint8ToBase64(slice))
  }
  return chunks
}

/** Canvas → Base64（经 data URL，兼容 WebView；小图或回退用） */
export const canvasToBase64 = (
  canvas: HTMLCanvasElement,
  format: ImageExportFormat,
): string => {
  const dataUrl =
    format.quality != null && format.mime !== 'image/png'
      ? canvas.toDataURL(format.mime, format.quality)
      : canvas.toDataURL(format.mime)

  if (!dataUrl.startsWith('data:')) {
    throw new Error(`无法编码为 ${format.label}`)
  }

  const comma = dataUrl.indexOf(',')
  if (comma < 0) {
    throw new Error(`无法编码为 ${format.label}`)
  }

  return dataUrl.slice(comma + 1)
}

export const exportBasename = (name: string): string => {
  const base = name.replace(/^.*[/\\]/, '').trim()
  return base || 'void-image'
}

/** 导出文件名主体（不含扩展名） */
export const sanitizeExportStem = (name: string): string =>
  (exportBasename(name).replace(/\.[^.]+$/, '') || 'void-image')
    .replace(/[/\\?%*:|"<>]/g, '-')
    .replace(/\.\./g, '-')
    .replace(/^\.+/, '') || 'void-image'

export const formatExportTimestamp = (): string => {
  const d = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return (
    `${d.getFullYear()}`
    + `${pad(d.getMonth() + 1)}`
    + `${pad(d.getDate())}`
    + `${pad(d.getHours())}`
    + `${pad(d.getMinutes())}`
    + `${pad(d.getSeconds())}`
  )
}

/** 通用导出：名称_时间戳.后缀 */
export const buildGeneralExportFilename = (exportName: string, ext: string): string =>
  `${sanitizeExportStem(exportName)}_${formatExportTimestamp()}.${ext}`

/** 缩略图 / ICO：名称_尺寸.后缀（无时间戳） */
export const buildSizedExportFilename = (
  exportName: string,
  size: number,
  ext: string,
): string => `${sanitizeExportStem(exportName)}_${size}x${size}.${ext}`

/** @deprecated 使用 buildGeneralExportFilename */
export const buildExportFilename = (baseName: string, format: ImageExportFormat): string =>
  buildGeneralExportFilename(baseName, format.ext)

export const ensureExportExtension = (path: string, ext: string): string => {
  const suffix = `.${ext.toLowerCase()}`
  if (path.toLowerCase().endsWith(suffix)) return path
  return `${path}${suffix}`
}

/** 将画布以 cover 方式裁为正方形 */
export const scaleCanvasToSquare = (
  canvas: HTMLCanvasElement,
  size: number,
): HTMLCanvasElement => {
  const edge = Math.max(1, Math.round(size))
  if (canvas.width === edge && canvas.height === edge) return canvas

  const scale = Math.max(edge / canvas.width, edge / canvas.height)
  const drawW = canvas.width * scale
  const drawH = canvas.height * scale
  const out = document.createElement('canvas')
  out.width = edge
  out.height = edge

  const ctx = out.getContext('2d')
  if (!ctx) return canvas
  ctx.imageSmoothingEnabled = true
  ctx.imageSmoothingQuality = 'high'
  ctx.drawImage(canvas, (edge - drawW) / 2, (edge - drawH) / 2, drawW, drawH)
  return out
}

/** 按目标尺寸缩放画布（等比例或自定义宽高） */
export const scaleCanvasToSize = (
  canvas: HTMLCanvasElement,
  size: ExportSizeOptions,
): HTMLCanvasElement => {
  const width = Math.max(1, Math.round(size.width))
  const height = Math.max(1, Math.round(size.height))
  if (width === canvas.width && height === canvas.height) return canvas

  const scaled = document.createElement('canvas')
  scaled.width = width
  scaled.height = height

  const ctx = scaled.getContext('2d')
  if (!ctx) return canvas
  ctx.imageSmoothingEnabled = true
  ctx.imageSmoothingQuality = 'high'
  ctx.drawImage(canvas, 0, 0, width, height)
  return scaled
}

/** 合并格式默认质量与压缩档位 */
export const resolveExportFormat = (
  format: ImageExportFormat,
  preset: ImageCompressionPreset,
): ImageExportFormat => {
  if (preset.quality == null || format.quality == null) return format
  return { ...format, quality: preset.quality }
}

export const formatInvokeError = (error: unknown): string => {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  if (error && typeof error === 'object') {
    const record = error as Record<string, unknown>
    if (typeof record.message === 'string') return record.message
  }
  return '操作失败'
}
