/**
 * image-editor-document.ts
 * 图片编辑器 — 多图会话与编辑状态快照
 */
import { base64ToBytes, uint8ToBase64 } from '@/utils/image-editor-export'
import type { SelectionRect, SelectionShape } from '@/utils/image-editor-crop'

export interface ImageEditState {
  originalPng: string
  workingPng: string
  cropCount: number
  width: number
  height: number
  originalWidth: number
  originalHeight: number
  selectionShape: SelectionShape
  selection: SelectionRect | null
}

export interface ImageDocumentEntry {
  id: string
  name: string
  file: File
  width: number
  height: number
  exportName: string
  editState: ImageEditState | null
  thumbUrl: string
}

export interface ImageDocumentSummary {
  id: string
  name: string
  thumbUrl: string
}

const canvasToPngBase64 = async (canvas: HTMLCanvasElement): Promise<string> => {
  const blob = await new Promise<Blob | null>((resolve) => {
    canvas.toBlob(resolve, 'image/png')
  })
  if (!blob || blob.size === 0) {
    throw new Error('无法捕获画布状态')
  }
  return uint8ToBase64(new Uint8Array(await blob.arrayBuffer()))
}

const loadPngBase64ToCanvas = async (
  canvas: HTMLCanvasElement,
  base64: string,
): Promise<void> => {
  const bytes = base64ToBytes(base64)
  const blob = new Blob([bytes.slice()], { type: 'image/png' })
  const bitmap = await createImageBitmap(blob)
  canvas.width = bitmap.width
  canvas.height = bitmap.height
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    bitmap.close()
    throw new Error('无法恢复画布')
  }
  ctx.drawImage(bitmap, 0, 0)
  bitmap.close()
}

export const captureEditState = async (
  original: HTMLCanvasElement,
  working: HTMLCanvasElement,
  cropCount: number,
  selectionShape: SelectionShape,
  selection: SelectionRect | null,
): Promise<ImageEditState> => ({
  originalPng: await canvasToPngBase64(original),
  workingPng: await canvasToPngBase64(working),
  cropCount,
  width: working.width,
  height: working.height,
  originalWidth: original.width,
  originalHeight: original.height,
  selectionShape,
  selection,
})

export const restoreEditState = async (
  original: HTMLCanvasElement,
  working: HTMLCanvasElement,
  state: ImageEditState,
): Promise<void> => {
  await loadPngBase64ToCanvas(original, state.originalPng)
  await loadPngBase64ToCanvas(working, state.workingPng)
}

export const createThumbUrl = async (
  working: HTMLCanvasElement,
  size = 48,
): Promise<string> => {
  const edge = Math.max(working.width, working.height, 1)
  const scale = size / edge
  const cssW = Math.max(1, Math.round(working.width * scale))
  const cssH = Math.max(1, Math.round(working.height * scale))

  const canvas = document.createElement('canvas')
  canvas.width = cssW
  canvas.height = cssH
  const ctx = canvas.getContext('2d')
  if (!ctx) return ''

  ctx.imageSmoothingEnabled = true
  ctx.imageSmoothingQuality = 'high'
  ctx.drawImage(working, 0, 0, cssW, cssH)

  const blob = await new Promise<Blob | null>((resolve) => {
    canvas.toBlob(resolve, 'image/png')
  })
  if (!blob) return ''
  return URL.createObjectURL(blob)
}

export const revokeThumbUrl = (url: string): void => {
  if (url) URL.revokeObjectURL(url)
}

export const createDocumentId = (): string => crypto.randomUUID()
