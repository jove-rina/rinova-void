/**
 * image-editor.ts
 * 图片编辑器 — Tauri API
 */
import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import { join } from '@tauri-apps/api/path'
import { open, save } from '@tauri-apps/plugin-dialog'
import { revealExportPath } from '@/api/export'
import {
  buildGeneralExportFilename,
  buildSizedExportFilename,
  bytesToBase64Chunks,
  canvasSupportsMime,
  canvasToBytes,
  compressionById,
  ensureExportExtension,
  EXPORT_SINGLE_SHOT_MAX_BYTES,
  formatById,
  ICO_MAX_SIZE,
  isValidIcoSize,
  resolveExportFormat,
  scaleCanvasToSize,
  scaleCanvasToSquare,
  uint8ToBase64,
  type ExportMode,
  type ExportSettings,
  type ImageCompressionPreset,
  type ImageExportFormat,
} from '@/utils/image-editor-export'
import type { ImageEditState } from '@/utils/image-editor-document'
import { isSquareCanvas, SQUARE_EXPORT_HINT } from '@/utils/image-editor-project'
import type { ImageEditorProjectSummary } from '@/utils/image-editor-project'

export interface ReadImageFileResult {
  name: string
  width: number
  height: number
  bytes: number[]
}

export interface EditorImageItem {
  id: string
  name: string
  width: number
  height: number
  bytes: number[]
  exportName: string
  editState: ImageEditState | null
}

export interface EditorSessionMeta {
  activeId: string
  exportMode: ExportMode
  exportFormatId: string
  compressionPresetId: string
  sizeScalePresetId: string
  thumbnailSizes: number[]
  icoSizes: number[]
  projectId: string | null
  projectName: string | null
}

export interface EditorSessionBatch {
  meta: EditorSessionMeta
  images: EditorImageItem[]
}

export interface ExportResult {
  paths: string[]
}

export const readImageFile = (path: string): Promise<ReadImageFileResult> =>
  invoke<ReadImageFileResult>('read_image_file', { path })

export const beginEditorSession = (
  sessionId: string,
  meta: EditorSessionMeta,
): Promise<void> =>
  invoke<void>('begin_editor_session', { sessionId, meta })

export const appendEditorSessionImage = (
  sessionId: string,
  item: EditorImageItem,
): Promise<void> =>
  invoke<void>('append_editor_session_image', { sessionId, item })

export const commitEditorSession = (sessionId: string): Promise<void> =>
  invoke<void>('commit_editor_session', { sessionId })

export const takeImageEditorSession = (): Promise<EditorSessionBatch | null> =>
  invoke<EditorSessionBatch | null>('take_image_editor_session')

export const prepareEditorSessionBatch = async (
  meta: EditorSessionMeta,
  images: Array<{
    id: string
    name: string
    width: number
    height: number
    exportName: string
    bytes: Uint8Array
    editState: ImageEditState | null
  }>,
): Promise<void> => {
  const sessionId = crypto.randomUUID()
  await beginEditorSession(sessionId, meta)
  try {
    for (const image of images) {
      await appendEditorSessionImage(sessionId, {
        id: image.id,
        name: image.name,
        width: image.width,
        height: image.height,
        exportName: image.exportName,
        bytes: Array.from(image.bytes),
        editState: image.editState,
      })
    }
    await commitEditorSession(sessionId)
  } catch (error) {
    throw error
  }
}

export const saveImageEditorProject = (json: string): Promise<string> =>
  invoke<string>('save_image_editor_project', { json })

export const listImageEditorProjects = (): Promise<ImageEditorProjectSummary[]> =>
  invoke<Array<{
    id: string
    name: string
    updatedAt: number
    imageCount: number
    previewBase64: string | null
  }>>('list_image_editor_projects').then((items) =>
    items.map((item) => ({
      id: item.id,
      name: item.name,
      updatedAt: item.updatedAt,
      imageCount: item.imageCount,
      previewBase64: item.previewBase64,
    })),
  )

export const loadImageEditorProject = (id: string): Promise<string> =>
  invoke<string>('load_image_editor_project', { id })

export const deleteImageEditorProject = (id: string): Promise<void> =>
  invoke<void>('delete_image_editor_project', { id })

export const openImageEditorWindow = (): Promise<void> =>
  invoke<void>('open_image_editor_window')

export const notifyImageEditorProjectsChanged = (): Promise<void> =>
  emit('image-editor-projects-changed')

const pickGeneralExportPath = async (
  exportName: string,
  format: ImageExportFormat,
): Promise<string | null> => {
  const defaultPath = buildGeneralExportFilename(exportName, format.ext)
  const picked = await save({
    defaultPath,
    filters: [{ name: format.label, extensions: [format.ext] }],
  })
  if (!picked) return null
  return ensureExportExtension(picked, format.ext)
}

const pickExportDirectory = async (): Promise<string | null> => {
  const dir = await open({
    directory: true,
    multiple: false,
    title: '选择导出目录',
  })
  if (!dir || Array.isArray(dir)) return null
  return dir
}

const exportBinaryBase64ToPath = (
  destPath: string,
  contentBase64: string,
): Promise<string> =>
  invoke<string>('export_binary_base64_to_path', {
    destPath,
    contentBase64,
  })

const convertImageBase64ToPath = (
  destPath: string,
  format: string,
  contentBase64: string,
): Promise<string> =>
  invoke<string>('convert_image_base64_to_path', {
    destPath,
    format,
    contentBase64,
  })

const beginExportBuffer = (exportId: string): Promise<void> =>
  invoke<void>('begin_export_buffer', { exportId })

const appendExportBase64 = (exportId: string, chunkBase64: string): Promise<void> =>
  invoke<void>('append_export_base64', { exportId, chunkBase64 })

const cancelExportBuffer = (exportId: string): Promise<void> =>
  invoke<void>('cancel_export_buffer', { exportId }).catch(() => {})

const finishExportBinary = (exportId: string, destPath: string): Promise<string> =>
  invoke<string>('finish_export_binary', { exportId, destPath })

const finishConvertExport = (
  exportId: string,
  destPath: string,
  format: string,
): Promise<string> =>
  invoke<string>('finish_convert_export', { exportId, destPath, format })

const uploadExportChunks = async (exportId: string, chunks: string[]): Promise<void> => {
  await beginExportBuffer(exportId)
  try {
    for (const chunk of chunks) {
      await appendExportBase64(exportId, chunk)
    }
  } catch (error) {
    await cancelExportBuffer(exportId)
    throw error
  }
}

const writeBytesToPath = async (
  destPath: string,
  bytes: Uint8Array,
  format: ImageExportFormat,
  targetFormat?: ImageExportFormat,
): Promise<string> => {
  const outputFormat = targetFormat ?? format

  if (bytes.length <= EXPORT_SINGLE_SHOT_MAX_BYTES) {
    const contentBase64 = uint8ToBase64(bytes)
    if (canvasSupportsMime(format.mime) && outputFormat.id === format.id) {
      return exportBinaryBase64ToPath(destPath, contentBase64)
    }
    return convertImageBase64ToPath(destPath, outputFormat.id, contentBase64)
  }

  const exportId = crypto.randomUUID()
  const chunks = bytesToBase64Chunks(bytes)
  await uploadExportChunks(exportId, chunks)

  if (canvasSupportsMime(format.mime) && outputFormat.id === format.id) {
    return finishExportBinary(exportId, destPath)
  }
  return finishConvertExport(exportId, destPath, outputFormat.id)
}

const exportGeneral = async (
  canvas: HTMLCanvasElement,
  settings: ExportSettings,
): Promise<string | null> => {
  const preset = compressionById(settings.compressionPresetId)
  const exportCanvas = scaleCanvasToSize(canvas, settings.exportSize)
  const format = resolveExportFormat(formatById(settings.formatId), preset)
  const destPath = await pickGeneralExportPath(settings.exportName, format)
  if (!destPath) return null

  const sourceFormat = canvasSupportsMime(format.mime) ? format : formatById('png')
  const bytes = await canvasToBytes(exportCanvas, sourceFormat)
  await writeBytesToPath(destPath, bytes, sourceFormat, format)
  return destPath
}

const exportThumbnailBatch = async (
  canvas: HTMLCanvasElement,
  settings: ExportSettings,
): Promise<string[] | null> => {
  if (settings.thumbnailSizes.length === 0) {
    throw new Error('请至少选择一个缩略图尺寸')
  }

  const dir = await pickExportDirectory()
  if (!dir) return null

  const preset = compressionById(settings.compressionPresetId)
  const format = resolveExportFormat(formatById(settings.formatId), preset)
  const pngFormat = formatById('png')
  const paths: string[] = []
  const sortedSizes = [...settings.thumbnailSizes].sort((a, b) => a - b)

  for (const size of sortedSizes) {
    const square = scaleCanvasToSquare(canvas, size)
    const bytes = await canvasToBytes(square, pngFormat)
    const filename = buildSizedExportFilename(settings.exportName, size, format.ext)
    const destPath = await join(dir, filename)
    const path = await writeBytesToPath(destPath, bytes, pngFormat, format)
    paths.push(path)
  }

  return paths
}

const exportIcoBatch = async (
  canvas: HTMLCanvasElement,
  settings: ExportSettings,
): Promise<string[] | null> => {
  const validSizes = settings.icoSizes.filter(isValidIcoSize)
  if (validSizes.length === 0) {
    throw new Error(`请至少选择一个 ICO 尺寸（最大 ${ICO_MAX_SIZE}×${ICO_MAX_SIZE}）`)
  }

  const dir = await pickExportDirectory()
  if (!dir) return null

  const pngFormat = formatById('png')
  const icoFormat = formatById('ico')
  const paths: string[] = []
  const sortedSizes = [...validSizes].sort((a, b) => a - b)

  for (const size of sortedSizes) {
    const square = scaleCanvasToSquare(canvas, size)
    const bytes = await canvasToBytes(square, pngFormat)
    const filename = buildSizedExportFilename(settings.exportName, size, icoFormat.ext)
    const destPath = await join(dir, filename)
    const path = await writeBytesToPath(destPath, bytes, pngFormat, icoFormat)
    paths.push(path)
  }

  return paths
}

/** 按导出设置导出；用户取消时返回 null */
export const exportImageWithSettings = async (
  canvas: HTMLCanvasElement,
  settings: ExportSettings,
): Promise<ExportResult | null> => {
  if (!canvas.width || !canvas.height) {
    throw new Error('画布尺寸无效')
  }

  if (!settings.exportName.trim()) {
    throw new Error('导出名称不能为空')
  }

  if (
    (settings.mode === 'thumbnail' || settings.mode === 'ico')
    && !isSquareCanvas(canvas.width, canvas.height)
  ) {
    throw new Error(SQUARE_EXPORT_HINT)
  }

  if (settings.mode === 'general') {
    const path = await exportGeneral(canvas, settings)
    if (!path) return null
    return { paths: [path] }
  }

  if (settings.mode === 'thumbnail') {
    const paths = await exportThumbnailBatch(canvas, settings)
    if (!paths) return null
    return { paths }
  }

  const paths = await exportIcoBatch(canvas, settings)
  if (!paths) return null
  return { paths }
}

export { revealExportPath, type ExportSettings, type ImageCompressionPreset, type ImageExportFormat }
