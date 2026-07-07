<script setup lang="ts">
/**
 * editor-session.vue
 * 图片编辑 — 左侧画布视口，右侧操作面板（对齐取色器布局）
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { Loader2, Minus, Plus, RotateCcw, Scan, Scissors, X } from '@lucide/vue'
import {
  clientToCanvasPixel,
  displayPercentToScale,
  fitCanvasTransform,
  formatDisplayZoomPercent,
  parseDisplayZoomPercent,
  zoomAtPoint,
  zoomScaleLimits,
  type CanvasTransform,
} from '@/utils/picker-canvas'
import {
  cropCanvas,
  normalizeSelection,
  type SelectionRect,
  type SelectionShape,
} from '@/utils/image-editor-crop'
import type { ImageExportFormat } from '@/utils/image-editor-export'

const props = defineProps<{
  file: File
  imageName: string
  imageWidth: number
  imageHeight: number
  exportFormatId: string
  exportFormats: ImageExportFormat[]
}>()

const emit = defineEmits<{
  'update:exportFormatId': [id: string]
  export: [canvas: HTMLCanvasElement]
  exit: []
}>()

const SHAPE_OPTIONS: { id: SelectionShape; label: string }[] = [
  { id: 'square', label: '正方形' },
  { id: 'rect', label: '矩形' },
  { id: 'circle', label: '圆形' },
  { id: 'ellipse', label: '椭圆' },
]

const viewportRef = ref<HTMLElement | null>(null)
const originalCanvasRef = ref<HTMLCanvasElement | null>(null)
const workingCanvasRef = ref<HTMLCanvasElement | null>(null)
const viewCanvasRef = ref<HTMLCanvasElement | null>(null)

const ready = ref(false)
const shape = ref<SelectionShape>('rect')
const selection = ref<SelectionRect | null>(null)
const draftSelection = ref<SelectionRect | null>(null)
const cropCount = ref(0)
const transform = ref<CanvasTransform>({ panX: 0, panY: 0, scale: 1 })
const baseScale = ref(1)
const isPanning = ref(false)
const isSelecting = ref(false)

const zoomInputValue = ref('100%')
const zoomInputFocused = ref(false)

let viewCtx: CanvasRenderingContext2D | null = null
let panStartX = 0
let panStartY = 0
let panOriginX = 0
let panOriginY = 0
let selectStart: { x: number; y: number } | null = null
let pointerDownX = 0
let pointerDownY = 0
let leftPointerDown = false
let resizeObserver: ResizeObserver | null = null
let cachedViewW = 0
let cachedViewH = 0
let cachedDpr = 1
let lastRenderedScale = -1

const CLICK_THRESHOLD = 6
const ZOOM_STEP_FACTOR = 1.08
const WHEEL_ZOOM_BASE = 1.035

const activeSelection = computed(() => draftSelection.value ?? selection.value)

const canvasPanStyle = computed(() => ({
  transform: `translate(${transform.value.panX}px, ${transform.value.panY}px)`,
}))

const viewportCursor = computed(() => {
  if (isPanning.value) return 'grabbing'
  return 'crosshair'
})

const overlayStyle = computed(() => {
  const sel = activeSelection.value
  if (!sel) return null
  const { panX, panY, scale } = transform.value
  return {
    left: `${panX + sel.x * scale}px`,
    top: `${panY + sel.y * scale}px`,
    width: `${sel.width * scale}px`,
    height: `${sel.height * scale}px`,
  }
})

const overlayShapeClass = computed(() => {
  if (shape.value === 'circle') return 'editor-session__overlay-shape--circle'
  if (shape.value === 'ellipse') return 'editor-session__overlay-shape--ellipse'
  return ''
})

const workingSizeLabel = computed(() => {
  const canvas = workingCanvasRef.value
  if (!canvas) return `${props.imageWidth} × ${props.imageHeight}`
  return `${canvas.width} × ${canvas.height}`
})

const selectionInfo = computed(() => {
  const sel = activeSelection.value
  if (!sel) return '未框选'
  return `X:${sel.x} Y:${sel.y} W:${sel.width} H:${sel.height}`
})

const canApplyCrop = computed(() => !!activeSelection.value)

const getViewportRect = (): DOMRect | null => viewportRef.value?.getBoundingClientRect() ?? null

const syncZoomInput = (): void => {
  if (!zoomInputFocused.value) {
    zoomInputValue.value = formatDisplayZoomPercent(transform.value.scale, baseScale.value)
  }
}

const getWorkingCanvas = (): HTMLCanvasElement | null => workingCanvasRef.value

const redrawCanvas = (): void => {
  const working = getWorkingCanvas()
  const view = viewCanvasRef.value
  if (!working || !view || !working.width || !working.height) return

  const { scale } = transform.value
  const dpr = window.devicePixelRatio || 1
  const cssW = Math.max(1, Math.round(working.width * scale))
  const cssH = Math.max(1, Math.round(working.height * scale))
  const pixelW = Math.round(cssW * dpr)
  const pixelH = Math.round(cssH * dpr)

  if (
    pixelW !== cachedViewW
    || pixelH !== cachedViewH
    || dpr !== cachedDpr
    || scale !== lastRenderedScale
  ) {
    view.width = pixelW
    view.height = pixelH
    view.style.width = `${cssW}px`
    view.style.height = `${cssH}px`
    cachedViewW = pixelW
    cachedViewH = pixelH
    cachedDpr = dpr
    lastRenderedScale = scale

    if (!viewCtx) viewCtx = view.getContext('2d')
    if (!viewCtx) return

    viewCtx.setTransform(dpr, 0, 0, dpr, 0, 0)
    viewCtx.clearRect(0, 0, cssW, cssH)
    viewCtx.imageSmoothingEnabled = scale <= baseScale.value
    viewCtx.drawImage(working, 0, 0, cssW, cssH)
  }
}

const fitToView = (): void => {
  const viewport = viewportRef.value
  const working = getWorkingCanvas()
  if (!viewport || !working || working.width <= 0) return
  transform.value = fitCanvasTransform(
    viewport.clientWidth,
    viewport.clientHeight,
    working.width,
    working.height,
  )
  baseScale.value = transform.value.scale
  lastRenderedScale = -1
  syncZoomInput()
  redrawCanvas()
}

const zoomLimits = (): { min: number; max: number } => zoomScaleLimits(baseScale.value)

const zoomBy = (factor: number): void => {
  const rect = getViewportRect()
  if (!rect) return
  const cx = rect.left + rect.width / 2
  const cy = rect.top + rect.height / 2
  const { min, max } = zoomLimits()
  transform.value = zoomAtPoint(transform.value, cx, cy, rect, factor, min, max)
  syncZoomInput()
  redrawCanvas()
}

const applyZoomFromInput = (): void => {
  const percent = parseDisplayZoomPercent(zoomInputValue.value)
  if (percent === null) {
    syncZoomInput()
    return
  }
  const rect = getViewportRect()
  if (!rect) return
  const { min, max } = zoomLimits()
  const nextScale = Math.min(max, Math.max(min, displayPercentToScale(percent, baseScale.value)))
  const factor = nextScale / transform.value.scale
  if (Math.abs(factor - 1) < 0.0001) {
    syncZoomInput()
    return
  }
  const cx = rect.left + rect.width / 2
  const cy = rect.top + rect.height / 2
  transform.value = zoomAtPoint(transform.value, cx, cy, rect, factor, min, max)
  syncZoomInput()
  redrawCanvas()
}

const onZoomInputFocus = (): void => {
  zoomInputFocused.value = true
}

const onZoomInputBlur = (): void => {
  zoomInputFocused.value = false
  applyZoomFromInput()
}

const onZoomInputKeydown = (event: KeyboardEvent): void => {
  if (event.key === 'Enter') {
    event.preventDefault()
    ;(event.target as HTMLInputElement).blur()
  }
}

const startPan = (clientX: number, clientY: number): void => {
  isPanning.value = true
  panStartX = clientX
  panStartY = clientY
  panOriginX = transform.value.panX
  panOriginY = transform.value.panY
}

const movePan = (clientX: number, clientY: number): void => {
  if (!isPanning.value) return
  transform.value = {
    ...transform.value,
    panX: panOriginX + (clientX - panStartX),
    panY: panOriginY + (clientY - panStartY),
  }
}

const endPan = (): void => {
  isPanning.value = false
}

const isPanButton = (button: number): boolean => button === 1 || button === 2

const updateDraftFromClient = (clientX: number, clientY: number): void => {
  const working = getWorkingCanvas()
  const rect = getViewportRect()
  if (!working || !rect || !selectStart) return

  const end = clientToCanvasPixel(
    clientX,
    clientY,
    rect,
    transform.value,
    working.width,
    working.height,
  )
  if (!end) {
    draftSelection.value = null
    return
  }

  draftSelection.value = normalizeSelection(
    selectStart,
    end,
    shape.value,
    working.width,
    working.height,
  )
}

const startSelect = (clientX: number, clientY: number): void => {
  const working = getWorkingCanvas()
  const rect = getViewportRect()
  if (!working || !rect) return

  const start = clientToCanvasPixel(
    clientX,
    clientY,
    rect,
    transform.value,
    working.width,
    working.height,
  )
  if (!start) return

  isSelecting.value = true
  selectStart = start
  draftSelection.value = null
  selection.value = null
}

const endSelect = (clientX: number, clientY: number): void => {
  if (!isSelecting.value) return
  updateDraftFromClient(clientX, clientY)
  selection.value = draftSelection.value
  draftSelection.value = null
  isSelecting.value = false
  selectStart = null
}

const onPointerDown = (event: PointerEvent): void => {
  if (isPanButton(event.button)) {
    event.preventDefault()
    viewportRef.value?.setPointerCapture(event.pointerId)
    startPan(event.clientX, event.clientY)
    return
  }
  if (event.button === 0) {
    leftPointerDown = true
    pointerDownX = event.clientX
    pointerDownY = event.clientY
    viewportRef.value?.setPointerCapture(event.pointerId)
    startSelect(event.clientX, event.clientY)
  }
}

const onPointerMove = (event: PointerEvent): void => {
  if (isPanning.value) {
    movePan(event.clientX, event.clientY)
    return
  }
  if (isSelecting.value) {
    updateDraftFromClient(event.clientX, event.clientY)
  }
}

const onPointerUp = (event: PointerEvent): void => {
  if (isPanning.value) {
    viewportRef.value?.releasePointerCapture(event.pointerId)
    endPan()
    return
  }

  if (event.button !== 0 || !leftPointerDown) return
  leftPointerDown = false
  viewportRef.value?.releasePointerCapture(event.pointerId)

  const dx = event.clientX - pointerDownX
  const dy = event.clientY - pointerDownY
  if (Math.hypot(dx, dy) <= CLICK_THRESHOLD) {
    endSelect(event.clientX, event.clientY)
    return
  }
  endSelect(event.clientX, event.clientY)
}

const onWheel = (event: WheelEvent): void => {
  event.preventDefault()
  const rect = getViewportRect()
  if (!rect) return
  const steps = Math.min(3, Math.max(1, Math.round(Math.abs(event.deltaY) / 50)))
  const factor =
    event.deltaY < 0
      ? WHEEL_ZOOM_BASE ** steps
      : 1 / WHEEL_ZOOM_BASE ** steps
  const { min, max } = zoomLimits()
  transform.value = zoomAtPoint(
    transform.value,
    event.clientX,
    event.clientY,
    rect,
    factor,
    min,
    max,
  )
  syncZoomInput()
  redrawCanvas()
}

const onContextMenu = (event: MouseEvent): void => {
  event.preventDefault()
}

const onKeyDown = (event: KeyboardEvent): void => {
  if (event.key === 'Escape') {
    event.preventDefault()
    emit('exit')
  } else if (event.key === '+' || event.key === '=') {
    event.preventDefault()
    zoomBy(ZOOM_STEP_FACTOR)
  } else if (event.key === '-') {
    event.preventDefault()
    zoomBy(1 / ZOOM_STEP_FACTOR)
  } else if (event.key === '0') {
    event.preventDefault()
    fitToView()
  }
}

const applyCrop = (): void => {
  const working = workingCanvasRef.value
  const original = originalCanvasRef.value
  const sel = activeSelection.value
  if (!working || !original || !sel) return

  const cropped = cropCanvas(working, sel, shape.value)
  working.width = cropped.width
  working.height = cropped.height
  const ctx = working.getContext('2d')
  ctx?.drawImage(cropped, 0, 0)
  cropCount.value += 1
  selection.value = null
  draftSelection.value = null
  lastRenderedScale = -1
  fitToView()
}

const resetToOriginal = (): void => {
  const working = workingCanvasRef.value
  const original = originalCanvasRef.value
  if (!working || !original) return

  working.width = original.width
  working.height = original.height
  const ctx = working.getContext('2d')
  ctx?.drawImage(original, 0, 0)
  cropCount.value = 0
  selection.value = null
  draftSelection.value = null
  lastRenderedScale = -1
  fitToView()
}

const clearSelection = (): void => {
  selection.value = null
  draftSelection.value = null
}

const requestExport = (): void => {
  const working = workingCanvasRef.value
  if (!working) return
  emit('export', working)
}

const loadImage = async (): Promise<void> => {
  await nextTick()
  const original = originalCanvasRef.value
  const working = workingCanvasRef.value
  if (!original || !working) return

  ready.value = false
  try {
    const bitmap = await createImageBitmap(props.file)
    original.width = bitmap.width
    original.height = bitmap.height
    working.width = bitmap.width
    working.height = bitmap.height

    const octx = original.getContext('2d')
    const wctx = working.getContext('2d')
    if (!octx || !wctx) {
      bitmap.close()
      return
    }

    octx.drawImage(bitmap, 0, 0)
    wctx.drawImage(bitmap, 0, 0)
    bitmap.close()

    ready.value = true
    cropCount.value = 0
    selection.value = null
    lastRenderedScale = -1
    fitToView()
  } catch {
    ready.value = false
  }
}

watch(shape, () => {
  if (isSelecting.value && selectStart) return
  selection.value = null
  draftSelection.value = null
})

watch(
  () => transform.value.scale,
  () => syncZoomInput(),
)

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)
  const viewport = viewportRef.value
  if (viewport) {
    resizeObserver = new ResizeObserver(() => redrawCanvas())
    resizeObserver.observe(viewport)
  }
  await loadImage()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  resizeObserver?.disconnect()
  resizeObserver = null
})
</script>

<template>
  <div class="editor-session">
    <div
      ref="viewportRef"
      class="editor-session__viewport"
      :class="{ 'editor-session__viewport--panning': isPanning }"
      :style="{ cursor: viewportCursor }"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @wheel="onWheel"
      @contextmenu="onContextMenu"
    >
      <canvas ref="originalCanvasRef" class="editor-session__source" aria-hidden="true" />
      <canvas ref="workingCanvasRef" class="editor-session__source" aria-hidden="true" />
      <canvas
        ref="viewCanvasRef"
        class="editor-session__canvas"
        :class="{ 'editor-session__canvas--ready': ready }"
        :style="canvasPanStyle"
      />
      <div
        v-if="activeSelection && overlayStyle"
        class="editor-session__overlay"
        :style="overlayStyle"
      >
        <div
          class="editor-session__overlay-shape"
          :class="overlayShapeClass"
        />
      </div>
      <div v-if="!ready" class="editor-session__loading">
        <Loader2 :size="32" :stroke-width="2" class="editor-session__loading-spin" />
        <p>正在加载图片…</p>
      </div>
    </div>

    <aside class="editor-session__panel">
      <header class="editor-session__panel-head">
        <h3 class="editor-session__panel-title">图片编辑</h3>
        <p class="editor-session__meta">{{ imageName }} · {{ workingSizeLabel }}</p>
      </header>

      <p class="editor-session__hint">
        左键拖拽框选 · 滚轮缩放 · 中键/右键拖拽平移 · Esc 退出
      </p>

      <section class="editor-session__section">
        <span class="editor-session__section-label">选区形状</span>
        <div class="editor-session__shape-group">
          <button
            v-for="opt in SHAPE_OPTIONS"
            :key="opt.id"
            type="button"
            class="editor-session__shape-btn"
            :class="{ 'editor-session__shape-btn--active': shape === opt.id }"
            @click="shape = opt.id"
          >
            {{ opt.label }}
          </button>
        </div>
      </section>

      <section class="editor-session__section editor-session__section--info">
        <span class="editor-session__section-label">选区信息</span>
        <span class="editor-session__info-value">{{ selectionInfo }}</span>
        <span v-if="cropCount > 0" class="editor-session__info-sub">
          已链式裁剪 {{ cropCount }} 次（原图已保留，可重置）
        </span>
      </section>

      <div class="editor-session__zoom">
        <span class="editor-session__zoom-label">视图缩放</span>
        <div class="editor-session__zoom-controls">
          <button
            type="button"
            class="editor-session__icon-btn"
            title="缩小 (-)"
            @click="zoomBy(1 / ZOOM_STEP_FACTOR)"
          >
            <Minus :size="14" :stroke-width="2" />
          </button>
          <div class="editor-session__zoom-input-wrap">
            <input
              v-model="zoomInputValue"
              type="text"
              class="editor-session__zoom-input"
              inputmode="decimal"
              aria-label="缩放比例"
              @focus="onZoomInputFocus"
              @blur="onZoomInputBlur"
              @keydown="onZoomInputKeydown"
            />
          </div>
          <button
            type="button"
            class="editor-session__icon-btn"
            title="放大 (+)"
            @click="zoomBy(ZOOM_STEP_FACTOR)"
          >
            <Plus :size="14" :stroke-width="2" />
          </button>
          <button
            type="button"
            class="editor-session__icon-btn"
            title="适应窗口 (0)"
            @click="fitToView"
          >
            <Scan :size="14" :stroke-width="2" />
          </button>
        </div>
      </div>

      <label class="editor-session__select-label">
        导出格式
        <select
          :value="exportFormatId"
          class="editor-session__select void-select"
          @change="emit('update:exportFormatId', ($event.target as HTMLSelectElement).value)"
        >
          <option v-for="fmt in exportFormats" :key="fmt.id" :value="fmt.id">
            {{ fmt.label }}
          </option>
        </select>
      </label>

      <div class="editor-session__actions editor-session__actions--stack">
        <button
          type="button"
          class="editor-session__btn editor-session__btn--primary"
          :disabled="!canApplyCrop"
          @click="applyCrop"
        >
          <Scissors :size="16" :stroke-width="2" />
          应用裁剪
        </button>
        <button
          type="button"
          class="editor-session__btn editor-session__btn--secondary"
          :disabled="cropCount === 0"
          @click="resetToOriginal"
        >
          <RotateCcw :size="16" :stroke-width="2" />
          重置为原图
        </button>
        <button
          type="button"
          class="editor-session__btn editor-session__btn--secondary"
          :disabled="!ready"
          @click="requestExport"
        >
          导出当前图片
        </button>
        <button
          v-if="activeSelection"
          type="button"
          class="editor-session__btn editor-session__btn--ghost"
          @click="clearSelection"
        >
          清除选区
        </button>
      </div>

      <div class="editor-session__actions">
        <button
          type="button"
          class="editor-session__btn editor-session__btn--exit"
          @click="emit('exit')"
        >
          <X :size="16" :stroke-width="2" />
          退出编辑
        </button>
      </div>
    </aside>
  </div>
</template>

<style lang="less" scoped>
.editor-session {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: #0a0a0c;
  display: flex;
  flex-direction: row;

  &__viewport {
    flex: 1;
    min-width: 0;
    position: relative;
    overflow: hidden;
    background: #000;
    touch-action: none;
    user-select: none;

    &--panning {
      cursor: grabbing;
    }
  }

  &__source {
    position: absolute;
    width: 0;
    height: 0;
    opacity: 0;
    pointer-events: none;
  }

  &__canvas {
    position: absolute;
    top: 0;
    left: 0;
    display: block;
    opacity: 0;
    transition: opacity 0.15s;
    will-change: transform;

    &--ready {
      opacity: 1;
    }
  }

  &__overlay {
    position: absolute;
    pointer-events: none;
    box-sizing: border-box;
  }

  &__overlay-shape {
    width: 100%;
    height: 100%;
    border: 2px solid #c084fc;
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.45);
    box-sizing: border-box;

    &--circle {
      border-radius: 50%;
    }

    &--ellipse {
      border-radius: 50%;
    }
  }

  &__loading {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    margin: 0;
    font-size: 13px;
    color: #9ca3af;
    pointer-events: none;
    background: rgba(0, 0, 0, 0.35);
  }

  &__loading-spin {
    color: #c084fc;
    animation: editor-spin 0.9s linear infinite;
  }

  &__panel {
    flex-shrink: 0;
    width: 300px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: rgba(22, 23, 29, 0.98);
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    overflow: hidden;
    min-height: 0;
  }

  &__panel-head {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
  }

  &__panel-title {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: #e5e7eb;
  }

  &__meta {
    margin: 0;
    font-size: 12px;
    color: #9ca3af;
    word-break: break-all;
  }

  &__hint {
    margin: 0;
    font-size: 11px;
    line-height: 1.5;
    color: #6b7280;
    flex-shrink: 0;
  }

  &__section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;

    &--info {
      gap: 4px;
    }
  }

  &__section-label {
    font-size: 12px;
    color: #9ca3af;
  }

  &__shape-group {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
  }

  &__shape-btn {
    padding: 8px 6px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;

    &--active {
      border-color: rgba(192, 132, 252, 0.5);
      background: rgba(192, 132, 252, 0.15);
      color: #e9d5ff;
    }

    &:hover:not(&--active) {
      background: rgba(255, 255, 255, 0.08);
    }
  }

  &__info-value {
    font-size: 12px;
    color: #e5e7eb;
    font-family: ui-monospace, 'Cascadia Code', monospace;
  }

  &__info-sub {
    font-size: 11px;
    color: #6b7280;
  }

  &__zoom {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex-shrink: 0;
  }

  &__zoom-label {
    font-size: 12px;
    color: #9ca3af;
  }

  &__zoom-controls {
    display: flex;
    align-items: stretch;
    gap: 6px;
    height: 32px;
  }

  &__zoom-input-wrap {
    flex: 1;
    min-width: 0;
    display: flex;
  }

  &__zoom-input {
    flex: 1;
    width: 100%;
    min-width: 0;
    height: 100%;
    padding: 0 8px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 13px;
    font-family: ui-monospace, 'Cascadia Code', monospace;
    text-align: center;
    box-sizing: border-box;

    &:focus {
      outline: none;
      border-color: rgba(192, 132, 252, 0.5);
    }
  }

  &__icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    cursor: pointer;

    &:hover {
      background: rgba(255, 255, 255, 0.08);
    }
  }

  &__select-label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    color: #9ca3af;
    flex-shrink: 0;
  }

  &__select {
    padding: 8px 10px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 13px;
  }

  &__actions {
    flex-shrink: 0;

    &--stack {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }

    &:last-child {
      margin-top: auto;
      padding-top: 8px;
    }
  }

  &__btn {
    width: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 16px;
    border-radius: 8px;
    border: none;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    &--primary {
      background: #c084fc;
      color: #000;
    }

    &--secondary {
      background: rgba(255, 255, 255, 0.08);
      color: #e5e7eb;
      border: 1px solid rgba(255, 255, 255, 0.12);
    }

    &--ghost {
      background: transparent;
      color: #9ca3af;
      border: 1px dashed rgba(255, 255, 255, 0.12);
      font-size: 12px;
      padding: 8px 12px;
    }

    &--exit {
      background: rgba(255, 255, 255, 0.08);
      color: #e5e7eb;
      border: 1px solid rgba(255, 255, 255, 0.12);
    }
  }
}

@keyframes editor-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
