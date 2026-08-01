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

export type ResizeHandle =
  | 'nw'
  | 'n'
  | 'ne'
  | 'w'
  | 'e'
  | 'sw'
  | 's'
  | 'se'

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

/** 判断点是否在选区外接矩形内 */
export const isPointInSelection = (point: DragPoint, selection: SelectionRect): boolean =>
  point.x >= selection.x
  && point.x <= selection.x + selection.width
  && point.y >= selection.y
  && point.y <= selection.y + selection.height

/** 平移选区并限制在画布内 */
export const translateSelection = (
  selection: SelectionRect,
  dx: number,
  dy: number,
  canvasWidth: number,
  canvasHeight: number,
): SelectionRect => {
  const x = clamp(selection.x + dx, 0, canvasWidth - selection.width)
  const y = clamp(selection.y + dy, 0, canvasHeight - selection.height)
  return {
    ...selection,
    x: Math.round(x),
    y: Math.round(y),
  }
}

/** 按拖拽手柄调整选区尺寸，正方形/圆形保持 1:1 */
export const resizeSelection = (
  origin: SelectionRect,
  handle: ResizeHandle,
  dx: number,
  dy: number,
  shape: SelectionShape,
  canvasWidth: number,
  canvasHeight: number,
): SelectionRect | null => {
  const left = origin.x
  const top = origin.y
  const right = origin.x + origin.width
  const bottom = origin.y + origin.height

  let x1 = left
  let y1 = top
  let x2 = right
  let y2 = bottom

  if (handle.includes('w')) x1 = left + dx
  if (handle.includes('e')) x2 = right + dx
  if (handle.includes('n')) y1 = top + dy
  if (handle.includes('s')) y2 = bottom + dy

  if (shape === 'square' || shape === 'circle') {
    let size = 1

    if (handle === 'nw') {
      size = Math.max(right - x1, bottom - y1, 1)
      x1 = right - size
      y1 = bottom - size
      x2 = right
      y2 = bottom
    } else if (handle === 'ne') {
      size = Math.max(x2 - left, bottom - y1, 1)
      x1 = left
      y1 = bottom - size
      x2 = left + size
      y2 = bottom
    } else if (handle === 'sw') {
      size = Math.max(right - x1, y2 - top, 1)
      x1 = right - size
      y1 = top
      x2 = right
      y2 = top + size
    } else if (handle === 'se') {
      size = Math.max(x2 - left, y2 - top, 1)
      x1 = left
      y1 = top
      x2 = left + size
      y2 = top + size
    } else if (handle === 'n') {
      size = Math.max(x2 - left, bottom - y1, 1)
      x1 = left
      y1 = bottom - size
      x2 = left + size
      y2 = bottom
    } else if (handle === 's') {
      size = Math.max(x2 - left, y2 - top, 1)
      x1 = left
      y1 = top
      x2 = left + size
      y2 = top + size
    } else if (handle === 'w') {
      size = Math.max(right - x1, y2 - top, 1)
      x1 = right - size
      y1 = top
      x2 = right
      y2 = top + size
    } else if (handle === 'e') {
      size = Math.max(x2 - left, y2 - top, 1)
      x1 = left
      y1 = top
      x2 = left + size
      y2 = top + size
    }
  } else {
    if (x1 > x2 - 1) x1 = x2 - 1
    if (y1 > y2 - 1) y1 = y2 - 1
  }

  x1 = clamp(Math.round(x1), 0, canvasWidth - 1)
  y1 = clamp(Math.round(y1), 0, canvasHeight - 1)
  x2 = clamp(Math.round(x2), x1 + 1, canvasWidth)
  y2 = clamp(Math.round(y2), y1 + 1, canvasHeight)

  const width = x2 - x1
  const height = y2 - y1
  if (width < 1 || height < 1) return null

  return { x: x1, y: y1, width, height }
}

/** 将选区限制在画布内，越界时裁剪；无效则返回 null */
export const clampSelectionToCanvas = (
  selection: SelectionRect | null | undefined,
  canvasWidth: number,
  canvasHeight: number,
): SelectionRect | null => {
  if (!selection || selection.width < 1 || selection.height < 1) return null

  const x = clamp(selection.x, 0, canvasWidth - 1)
  const y = clamp(selection.y, 0, canvasHeight - 1)
  const width = Math.min(selection.width, canvasWidth - x)
  const height = Math.min(selection.height, canvasHeight - y)

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
