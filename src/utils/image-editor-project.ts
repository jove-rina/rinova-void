/**
 * image-editor-project.ts
 * 图片编辑项目持久化与正方形校验
 */
import type { ImageEditState } from '@/utils/image-editor-document'
import { uint8ToBase64 } from '@/utils/image-editor-export'
import type { ExportMode } from '@/utils/image-editor-export'

export interface ImageEditorProjectDocument {
  id: string
  name: string
  width: number
  height: number
  exportName: string
  editState: ImageEditState | null
  sourceBase64: string
}

export interface ImageEditorProject {
  id: string
  name: string
  updatedAt: number
  activeDocumentId: string
  exportMode: ExportMode
  exportFormatId: string
  compressionPresetId: string
  sizeScalePresetId: string
  thumbnailSizes: number[]
  icoSizes: number[]
  previewBase64: string | null
  documents: ImageEditorProjectDocument[]
}

export interface ImageEditorProjectSummary {
  id: string
  name: string
  updatedAt: number
  imageCount: number
  previewBase64: string | null
}

export const isSquareCanvas = (width: number, height: number): boolean =>
  width > 0 && width === height

export const SQUARE_EXPORT_HINT =
  '缩略图与 ICO 要求正方形图片。请使用正方形选区并确认裁剪，或先裁剪为 1:1 比例'

export const fileToBase64 = async (file: File): Promise<string> =>
  uint8ToBase64(new Uint8Array(await file.arrayBuffer()))

export const createProjectId = (): string => crypto.randomUUID()

export const formatProjectTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`
}

export const defaultProjectName = (): string => {
  const date = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return `图片项目 ${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())} ${pad(date.getHours())}${pad(date.getMinutes())}`
}

export const sanitizeProjectName = (name: string): string => {
  const trimmed = name.trim().replace(/[/\\?%*:|"<>]/g, '-')
  return trimmed.slice(0, 80)
}

export const parseProjectJson = (json: string): ImageEditorProject => {
  const data = JSON.parse(json) as ImageEditorProject
  if (!data.id || !Array.isArray(data.documents)) {
    throw new Error('项目数据无效')
  }
  return data
}

export const summaryFromRust = (item: {
  id: string
  name: string
  updatedAt: number
  imageCount: number
  previewBase64: string | null
}): ImageEditorProjectSummary => ({
  id: item.id,
  name: item.name,
  updatedAt: item.updatedAt,
  imageCount: item.imageCount,
  previewBase64: item.previewBase64,
})
