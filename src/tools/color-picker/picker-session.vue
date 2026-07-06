<script setup lang="ts">
/**
 * picker-session.vue
 * 截屏取色 — 左侧快照视口，右侧操作面板（多点取色）
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Loader2, Minus, Plus, RefreshCw, Scan, X } from '@lucide/vue'
import {
  gridMeta,
  MAGNIFY_OPTIONS,
  type MonitorInfo,
  type StartPickerResult,
} from '@/api/color-picker'
import ColorRecordsSection from './color-records-section.vue'
import {
  clientToCanvasPixel,
  decodeBase64PngToImageBitmap,
  fitCanvasTransform,
  sampleCanvasPixel,
  zoomAtPoint,
  type CanvasTransform,
} from '@/utils/picker-canvas'
import { toHex, toHsl, toRgb } from '@/utils/color-format'
import type { ColorRecord, ColorRecordExportFormat } from '@/utils/color-records'

const props = defineProps<{
  session: StartPickerResult
  records: ColorRecord[]
  monitors: MonitorInfo[]
  refreshing: boolean
}>()

const emit = defineEmits<{
  pick: [rgb: [number, number, number]]
  rename: [id: string, name: string]
  'delete-record': [id: string]
  'copy-hex': [record: ColorRecord]
  'copy-rgb': [record: ColorRecord]
  'copy-hsl': [record: ColorRecord]
  export: [format: ColorRecordExportFormat]
  refresh: [monitorIndex: number, captureAll: boolean, radius: number]
  'update:radius': [radius: number]
  'load-error': [message: string]
  exit: []
}>()

const resolveMonitorIndex = (session: StartPickerResult): number => {
  if (session.capture_all) {
    return props.monitors[0]?.index ?? 0
  }
  const found = props.monitors.find((mon) => mon.index === session.monitor_index)
  return found?.index ?? props.monitors[0]?.index ?? 0
}

const localMonitorIndex = ref(resolveMonitorIndex(props.session))
const localCaptureAll = ref(props.session.capture_all)

const viewportRef = ref<HTMLElement | null>(null)
const sourceCanvasRef = ref<HTMLCanvasElement | null>(null)
const viewCanvasRef = ref<HTMLCanvasElement | null>(null)
const activeRadius = ref(props.session.radius)
const previewCenter = ref<[number, number, number]>([0, 0, 0])
const previewPixels = ref<[number, number, number][]>([])
const ready = ref(false)
const transform = ref<CanvasTransform>({ panX: 0, panY: 0, scale: 1 })
const isPanning = ref(false)

let sourceCtx: CanvasRenderingContext2D | null = null
let viewCtx: CanvasRenderingContext2D | null = null
let panStartX = 0
let panStartY = 0
let panOriginX = 0
let panOriginY = 0
let pointerDownX = 0
let pointerDownY = 0
let leftPointerDown = false
let loadGeneration = 0
let resizeObserver: ResizeObserver | null = null

const CLICK_THRESHOLD = 6

const magnifyEnabled = computed(() => activeRadius.value > 0)
const magnifyGrid = computed(() => gridMeta(activeRadius.value))
const gridSize = computed(() => magnifyGrid.value.gridSize)
const centerIndex = computed(() => magnifyGrid.value.centerIndex)
const zoomPercent = computed(() => `${Math.round(transform.value.scale * 100)}%`)

const viewportCursor = computed(() => (isPanning.value ? 'grabbing' : 'crosshair'))

const previewHex = computed(() => {
  const [r, g, b] = previewCenter.value
  return toHex(r, g, b)
})
const previewRgb = computed(() => {
  const [r, g, b] = previewCenter.value
  return toRgb(r, g, b)
})
const previewHsl = computed(() => {
  const [r, g, b] = previewCenter.value
  return toHsl(r, g, b)
})
const previewSwatchStyle = computed(() => ({ background: previewHex.value }))

const cellStyle = (px: [number, number, number]): { background: string } => ({
  background: toRgb(px[0], px[1], px[2]),
})

const getViewportRect = (): DOMRect | null => viewportRef.value?.getBoundingClientRect() ?? null

const renderView = (): void => {
  const viewport = viewportRef.value
  const source = sourceCanvasRef.value
  const view = viewCanvasRef.value
  if (!viewport || !source || !view || !source.width || !source.height) return

  const vw = viewport.clientWidth
  const vh = viewport.clientHeight
  if (vw <= 0 || vh <= 0) return

  const dpr = window.devicePixelRatio || 1
  view.width = Math.round(vw * dpr)
  view.height = Math.round(vh * dpr)
  view.style.width = `${vw}px`
  view.style.height = `${vh}px`

  if (!viewCtx) {
    viewCtx = view.getContext('2d')
  }
  if (!viewCtx) return

  const { panX, panY, scale } = transform.value
  viewCtx.setTransform(dpr, 0, 0, dpr, 0, 0)
  viewCtx.imageSmoothingEnabled = scale < 1
  viewCtx.clearRect(0, 0, vw, vh)
  viewCtx.drawImage(
    source,
    -panX / scale,
    -panY / scale,
    vw / scale,
    vh / scale,
    0,
    0,
    vw,
    vh,
  )
}

const fitToView = (): void => {
  const viewport = viewportRef.value
  const source = sourceCanvasRef.value
  if (!viewport || !source || source.width <= 0) return
  transform.value = fitCanvasTransform(
    viewport.clientWidth,
    viewport.clientHeight,
    source.width,
    source.height,
  )
  renderView()
}

const sampleAtClient = (clientX: number, clientY: number): boolean => {
  const source = sourceCanvasRef.value
  const rect = getViewportRect()
  if (!source || !sourceCtx || !rect) return false

  const pos = clientToCanvasPixel(
    clientX,
    clientY,
    rect,
    transform.value,
    source.width,
    source.height,
  )
  if (!pos) return false

  const sample = sampleCanvasPixel(
    sourceCtx,
    pos.x,
    pos.y,
    activeRadius.value,
    source.width,
    source.height,
  )
  previewCenter.value = sample.center
  previewPixels.value = sample.pixels
  return true
}

const zoomBy = (factor: number): void => {
  const rect = getViewportRect()
  if (!rect) return
  const cx = rect.left + rect.width / 2
  const cy = rect.top + rect.height / 2
  transform.value = zoomAtPoint(transform.value, cx, cy, rect, factor)
  renderView()
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
  renderView()
}

const endPan = (): void => {
  isPanning.value = false
}

const isPanButton = (button: number): boolean => button === 1 || button === 2

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
    sampleAtClient(event.clientX, event.clientY)
  }
}

const onPointerMove = (event: PointerEvent): void => {
  if (isPanning.value) {
    movePan(event.clientX, event.clientY)
    return
  }
  sampleAtClient(event.clientX, event.clientY)
}

const onPointerUp = (event: PointerEvent): void => {
  if (isPanning.value) {
    viewportRef.value?.releasePointerCapture(event.pointerId)
    endPan()
    return
  }

  if (event.button !== 0 || !leftPointerDown) return
  leftPointerDown = false

  const dx = event.clientX - pointerDownX
  const dy = event.clientY - pointerDownY
  if (Math.hypot(dx, dy) > CLICK_THRESHOLD) return

  if (sampleAtClient(event.clientX, event.clientY)) {
    emit('pick', previewCenter.value)
  }
}

const onWheel = (event: WheelEvent): void => {
  event.preventDefault()
  const rect = getViewportRect()
  if (!rect) return
  const factor = event.deltaY < 0 ? 1.12 : 1 / 1.12
  transform.value = zoomAtPoint(
    transform.value,
    event.clientX,
    event.clientY,
    rect,
    factor,
  )
  renderView()
}

const onContextMenu = (event: MouseEvent): void => {
  event.preventDefault()
}

const onKeyDown = (event: KeyboardEvent): void => {
  if (event.key === 'Escape') {
    event.preventDefault()
    emit('exit')
  } else if (event.key === 'm' || event.key === 'M') {
    event.preventDefault()
    const idx = MAGNIFY_OPTIONS.findIndex((o) => o.radius === activeRadius.value)
    activeRadius.value = MAGNIFY_OPTIONS[(idx + 1) % MAGNIFY_OPTIONS.length].radius
  } else if (event.key === '+' || event.key === '=') {
    event.preventDefault()
    zoomBy(1.2)
  } else if (event.key === '-') {
    event.preventDefault()
    zoomBy(1 / 1.2)
  } else if (event.key === '0') {
    event.preventDefault()
    fitToView()
  }
}

watch(activeRadius, (radius) => {
  emit('update:radius', radius)
  const rect = getViewportRect()
  if (rect) {
    sampleAtClient(rect.left + rect.width / 2, rect.top + rect.height / 2)
  }
})

const loadSnapshot = async (session: StartPickerResult): Promise<void> => {
  await nextTick()
  const source = sourceCanvasRef.value
  const viewport = viewportRef.value
  if (!source || !viewport) {
    emit('load-error', '快照视图未就绪')
    return
  }

  const generation = ++loadGeneration
  ready.value = false

  try {
    const bitmap = await decodeBase64PngToImageBitmap(session.image_base64)
    if (generation !== loadGeneration) {
      bitmap.close()
      return
    }

    source.width = bitmap.width
    source.height = bitmap.height
    sourceCtx = source.getContext('2d', { willReadFrequently: true })
    if (!sourceCtx) {
      bitmap.close()
      emit('load-error', '无法创建 Canvas 上下文')
      return
    }

    sourceCtx.drawImage(bitmap, 0, 0)
    bitmap.close()
    ready.value = true
    fitToView()
    const rect = viewport.getBoundingClientRect()
    sampleAtClient(rect.left + rect.width / 2, rect.top + rect.height / 2)
  } catch {
    if (generation !== loadGeneration) return
    ready.value = false
    emit('load-error', '快照加载失败')
  }
}

const requestRefresh = (): void => {
  if (props.refreshing) return
  emit('refresh', localMonitorIndex.value, localCaptureAll.value, activeRadius.value)
}

const onMonitorChange = (): void => {
  if (localCaptureAll.value) return
  requestRefresh()
}

const onCaptureAllChange = (): void => {
  requestRefresh()
}

watch(
  () => props.session,
  (session) => {
    localCaptureAll.value = session.capture_all
    if (!session.capture_all) {
      localMonitorIndex.value = resolveMonitorIndex(session)
    }
    activeRadius.value = session.radius
  },
)

watch(
  () => props.session.image_base64,
  () => {
    void loadSnapshot(props.session)
  },
)

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)
  await getCurrentWindow().setFullscreen(true)

  const viewport = viewportRef.value
  if (viewport) {
    resizeObserver = new ResizeObserver(() => {
      renderView()
    })
    resizeObserver.observe(viewport)
  }

  await loadSnapshot(props.session)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  resizeObserver?.disconnect()
  resizeObserver = null
})
</script>

<template>
  <div class="picker-session">
    <div
      ref="viewportRef"
      class="picker-session__viewport"
      :class="{ 'picker-session__viewport--panning': isPanning }"
      :style="{ cursor: viewportCursor }"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @wheel="onWheel"
      @contextmenu="onContextMenu"
    >
      <canvas ref="sourceCanvasRef" class="picker-session__source" aria-hidden="true" />
      <canvas
        ref="viewCanvasRef"
        class="picker-session__canvas"
        :class="{ 'picker-session__canvas--ready': ready }"
      />
      <div v-if="!ready || refreshing" class="picker-session__loading">
        <Loader2 :size="32" :stroke-width="2" class="picker-session__loading-spin" />
        <p>{{ refreshing ? '正在刷新屏幕…' : '正在加载快照…' }}</p>
      </div>
    </div>

    <aside class="picker-session__panel">
      <header class="picker-session__panel-head">
        <h3 class="picker-session__panel-title">取色</h3>
        <p class="picker-session__monitor">{{ session.monitor_label }}</p>
      </header>

      <p class="picker-session__hint">
        左键点击取色 · 滚轮缩放 · 中键/右键拖拽 · Esc 退出
      </p>

      <section class="picker-session__capture">
        <label class="picker-session__select-label">
          截取屏幕
          <select
            v-model.number="localMonitorIndex"
            class="picker-session__select void-select"
            :disabled="refreshing || localCaptureAll || !monitors.length"
            @change="onMonitorChange"
          >
            <option v-for="mon in monitors" :key="mon.index" :value="mon.index">
              {{ mon.label }}
            </option>
          </select>
        </label>

        <label class="picker-session__option">
          <input
            v-model="localCaptureAll"
            type="checkbox"
            :disabled="refreshing"
            @change="onCaptureAllChange"
          />
          截取全部屏幕
        </label>

        <button
          type="button"
          class="picker-session__refresh-btn"
          :disabled="refreshing"
          @click="requestRefresh"
        >
          <Loader2 v-if="refreshing" :size="14" :stroke-width="2" class="picker-session__refresh-spin" />
          <RefreshCw v-else :size="14" :stroke-width="2" />
          刷新屏幕
        </button>
      </section>

      <div v-if="magnifyEnabled" class="picker-session__grid-wrap">
        <div class="picker-session__grid" :style="{ '--grid-size': gridSize }">
          <div
            v-for="(px, index) in previewPixels"
            :key="index"
            class="picker-session__cell"
            :class="{ 'picker-session__cell--center': index === centerIndex }"
            :style="cellStyle(px)"
          />
        </div>
      </div>
      <div v-else class="picker-session__swatch" :style="previewSwatchStyle" />

      <div class="picker-session__values">
        <span class="picker-session__hex">{{ previewHex }}</span>
        <span class="picker-session__rgb">{{ previewRgb }}</span>
        <span class="picker-session__hsl">{{ previewHsl }}</span>
      </div>

      <div class="picker-session__zoom">
        <span class="picker-session__zoom-label">视图缩放</span>
        <div class="picker-session__zoom-controls">
          <button
            type="button"
            class="picker-session__icon-btn"
            title="缩小 (-)"
            @click="zoomBy(1 / 1.2)"
          >
            <Minus :size="14" :stroke-width="2" />
          </button>
          <span class="picker-session__zoom-value">{{ zoomPercent }}</span>
          <button
            type="button"
            class="picker-session__icon-btn"
            title="放大 (+)"
            @click="zoomBy(1.2)"
          >
            <Plus :size="14" :stroke-width="2" />
          </button>
          <button
            type="button"
            class="picker-session__icon-btn"
            title="适应窗口 (0)"
            @click="fitToView"
          >
            <Scan :size="14" :stroke-width="2" />
          </button>
        </div>
      </div>

      <label class="picker-session__select-label">
        放大倍数
        <select v-model.number="activeRadius" class="picker-session__select void-select">
          <option v-for="opt in MAGNIFY_OPTIONS" :key="opt.radius" :value="opt.radius">
            {{ opt.label }}
          </option>
        </select>
      </label>

      <ColorRecordsSection
        :records="records"
        :collapsible="false"
        fill
        empty-text="点击左侧画面开始取色"
        @rename="(id, name) => emit('rename', id, name)"
        @delete-record="(id) => emit('delete-record', id)"
        @copy-hex="(record) => emit('copy-hex', record)"
        @copy-rgb="(record) => emit('copy-rgb', record)"
        @copy-hsl="(record) => emit('copy-hsl', record)"
        @export="(format) => emit('export', format)"
      />

      <div class="picker-session__actions">
        <button
          type="button"
          class="picker-session__btn picker-session__btn--exit"
          @click="emit('exit')"
        >
          <X :size="16" :stroke-width="2" />
          退出取色
        </button>
      </div>
    </aside>
  </div>
</template>

<style lang="less" scoped>
.picker-session {
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
    inset: 0;
    display: block;
    width: 100%;
    height: 100%;
    opacity: 0;
    transition: opacity 0.15s;

    &--ready {
      opacity: 1;
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
    animation: picker-spin 0.9s linear infinite;
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

  &__monitor {
    margin: 0;
    font-size: 12px;
    color: #9ca3af;
  }

  &__hint {
    margin: 0;
    font-size: 11px;
    line-height: 1.5;
    color: #6b7280;
    flex-shrink: 0;
  }

  &__capture {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
  }

  &__option {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #9ca3af;
    cursor: pointer;
    user-select: none;

    input {
      accent-color: #c084fc;
    }
  }

  &__refresh-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px 12px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }

    &:hover:not(:disabled) {
      background: rgba(255, 255, 255, 0.08);
    }
  }

  &__refresh-spin {
    animation: picker-spin 0.9s linear infinite;
  }

  &__grid-wrap {
    overflow: auto;
    max-width: 100%;
    flex-shrink: 0;
  }

  &__grid {
    display: grid;
    grid-template-columns: repeat(var(--grid-size), 12px);
    grid-template-rows: repeat(var(--grid-size), 12px);
    gap: 1px;
    padding: 2px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
  }

  &__cell {
    width: 12px;
    height: 12px;

    &--center {
      outline: 2px solid #fff;
      outline-offset: -1px;
    }
  }

  &__swatch {
    width: 56px;
    height: 56px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    flex-shrink: 0;
  }

  &__values {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
  }

  &__hex {
    font-size: 16px;
    font-weight: 600;
    color: #e5e7eb;
    font-family: ui-monospace, 'Cascadia Code', monospace;
  }

  &__rgb {
    font-size: 12px;
    color: #9ca3af;
    font-family: ui-monospace, 'Cascadia Code', monospace;
  }

  &__hsl {
    font-size: 11px;
    color: #6b7280;
    font-family: ui-monospace, 'Cascadia Code', monospace;
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
    align-items: center;
    gap: 6px;
  }

  &__zoom-value {
    flex: 1;
    text-align: center;
    font-size: 13px;
    color: #e5e7eb;
    font-family: ui-monospace, 'Cascadia Code', monospace;
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
    margin-top: auto;
    padding-top: 8px;
    flex-shrink: 0;
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

    &--exit {
      background: rgba(255, 255, 255, 0.08);
      color: #e5e7eb;
      border: 1px solid rgba(255, 255, 255, 0.12);
    }
  }
}

@keyframes picker-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
