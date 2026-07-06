/**
 * picker-canvas.ts
 * 截屏 Canvas 坐标映射与像素采样
 */
export interface CanvasSample {
  pixels: [number, number, number][]
  center: [number, number, number]
}

export interface CanvasTransform {
  panX: number
  panY: number
  scale: number
}

const readRgb = (data: Uint8ClampedArray, idx: number): [number, number, number] => [
  data[idx] ?? 0,
  data[idx + 1] ?? 0,
  data[idx + 2] ?? 0,
]

/** Base64 PNG → bytes（供测试与解码复用） */
export const decodeBase64ToBytes = (base64: string): Uint8Array => {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}

/**
 * 将 Rust 返回的 PNG base64 解码为 ImageBitmap。
 * 避免 `data:` URL — 生产构建 CSP 默认禁止 img-src data:，会导致快照永远加载失败。
 */
export const decodeBase64PngToImageBitmap = async (base64: string): Promise<ImageBitmap> => {
  const bytes = decodeBase64ToBytes(base64)
  const buffer = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer
  const blob = new Blob([buffer], { type: 'image/png' })
  return createImageBitmap(blob)
}

/** 视口坐标 → Canvas 像素（含平移缩放） */
export const clientToCanvasPixel = (
  clientX: number,
  clientY: number,
  viewportRect: DOMRect,
  transform: CanvasTransform,
  canvasWidth: number,
  canvasHeight: number,
): { x: number; y: number } | null => {
  const localX = clientX - viewportRect.left
  const localY = clientY - viewportRect.top
  const x = Math.floor((localX - transform.panX) / transform.scale)
  const y = Math.floor((localY - transform.panY) / transform.scale)

  if (x < 0 || y < 0 || x >= canvasWidth || y >= canvasHeight) {
    return null
  }
  return { x, y }
}

/** 计算适应视口的初始变换 */
export const fitCanvasTransform = (
  viewportWidth: number,
  viewportHeight: number,
  canvasWidth: number,
  canvasHeight: number,
): CanvasTransform => {
  if (viewportWidth <= 0 || viewportHeight <= 0 || canvasWidth <= 0 || canvasHeight <= 0) {
    return { panX: 0, panY: 0, scale: 1 }
  }
  const scale = Math.min(viewportWidth / canvasWidth, viewportHeight / canvasHeight, 1)
  return {
    scale,
    panX: (viewportWidth - canvasWidth * scale) / 2,
    panY: (viewportHeight - canvasHeight * scale) / 2,
  }
}

/** 以光标为中心缩放 */
export const zoomAtPoint = (
  transform: CanvasTransform,
  clientX: number,
  clientY: number,
  viewportRect: DOMRect,
  factor: number,
  minScale = 0.1,
  maxScale = 8,
): CanvasTransform => {
  const localX = clientX - viewportRect.left
  const localY = clientY - viewportRect.top
  const nextScale = Math.min(maxScale, Math.max(minScale, transform.scale * factor))
  const ratio = nextScale / transform.scale
  return {
    scale: nextScale,
    panX: localX - (localX - transform.panX) * ratio,
    panY: localY - (localY - transform.panY) * ratio,
  }
}

/** 从 Canvas 指定像素采样（含放大镜区域） */
export const sampleCanvasPixel = (
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  radius: number,
  canvasWidth: number,
  canvasHeight: number,
): CanvasSample => {
  if (radius <= 0) {
    const data = ctx.getImageData(x, y, 1, 1).data
    const center = readRgb(data, 0)
    return { pixels: [center], center }
  }

  const r = radius
  const left = Math.max(0, x - r)
  const top = Math.max(0, y - r)
  const right = Math.min(canvasWidth - 1, x + r)
  const bottom = Math.min(canvasHeight - 1, y + r)
  const w = right - left + 1
  const h = bottom - top + 1
  const data = ctx.getImageData(left, top, w, h).data

  const side = r * 2 + 1
  const pixels: [number, number, number][] = []
  for (let dy = -r; dy <= r; dy++) {
    for (let dx = -r; dx <= r; dx++) {
      const px = x + dx
      const py = y + dy
      if (px < 0 || py < 0 || px >= canvasWidth || py >= canvasHeight) {
        pixels.push([0, 0, 0])
        continue
      }
      const sx = px - left
      const sy = py - top
      const idx = (sy * w + sx) * 4
      pixels.push(readRgb(data, idx))
    }
  }

  const centerIdx = r * side + r
  return { pixels, center: pixels[centerIdx] ?? [0, 0, 0] }
}
