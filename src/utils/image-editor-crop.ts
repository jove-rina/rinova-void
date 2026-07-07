/**
 * image-editor-crop.ts
 * 图片编辑器 — 选区几何与裁剪
 */
export type SelectionShape = 'square' | 'rect' | 'circle' | 'ellipse'

export interface SelectionRect {
  x: number
  y: number
  width: number
  height: number
}

export interface DragPoint {
  x: number
  y: number
}

const clamp = (value: number, min: number, max: number): number =>
  Math.min(max, Math.max(min, value))

/** 将拖拽起止点规范为画布内的选区外接矩形 */
export const normalizeSelection = (
  start: DragPoint,
  end: DragPoint,
  shape: SelectionShape,
  canvasWidth: number,
  canvasHeight: number,
): SelectionRect | null => {
  let dx = end.x - start.x
  let dy = end.y - start.y

  if (shape === 'square' || shape === 'circle') {
    const size = Math.max(Math.abs(dx), Math.abs(dy))
    dx = dx < 0 ? -size : size
    dy = dy < 0 ? -size : size
  }

  let x = dx < 0 ? start.x + dx : start.x
  let y = dy < 0 ? start.y + dy : start.y
  let width = Math.abs(dx)
  let height = Math.abs(dy)

  if (width < 1 || height < 1) return null

  x = clamp(x, 0, canvasWidth - 1)
  y = clamp(y, 0, canvasHeight - 1)
  width = Math.min(width, canvasWidth - x)
  height = Math.min(height, canvasHeight - y)

  if (width < 1 || height < 1) return null

  return {
    x: Math.round(x),
    y: Math.round(y),
    width: Math.round(width),
    height: Math.round(height),
  }
}

/** 在画布上按形状裁剪，返回新 canvas */
export const cropCanvas = (
  source: HTMLCanvasElement,
  selection: SelectionRect,
  shape: SelectionShape,
): HTMLCanvasElement => {
  const out = document.createElement('canvas')
  out.width = selection.width
  out.height = selection.height
  const ctx = out.getContext('2d')
  if (!ctx) return out

  if (shape === 'rect' || shape === 'square') {
    ctx.drawImage(
      source,
      selection.x,
      selection.y,
      selection.width,
      selection.height,
      0,
      0,
      selection.width,
      selection.height,
    )
    return out
  }

  const cx = selection.width / 2
  const cy = selection.height / 2
  ctx.save()
  ctx.beginPath()
  if (shape === 'circle') {
    const r = Math.min(selection.width, selection.height) / 2
    ctx.arc(cx, cy, r, 0, Math.PI * 2)
  } else {
    ctx.ellipse(cx, cy, selection.width / 2, selection.height / 2, 0, 0, Math.PI * 2)
  }
  ctx.clip()
  ctx.drawImage(
    source,
    selection.x,
    selection.y,
    selection.width,
    selection.height,
    0,
    0,
    selection.width,
    selection.height,
  )
  ctx.restore()
  return out
}

/** 将 canvas 像素复制到目标 canvas */
export const copyCanvas = (source: HTMLCanvasElement): HTMLCanvasElement => {
  const out = document.createElement('canvas')
  out.width = source.width
  out.height = source.height
  const ctx = out.getContext('2d')
  if (ctx) {
    ctx.drawImage(source, 0, 0)
  }
  return out
}
