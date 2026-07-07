<script setup lang="ts">
/**
 * editor-session.vue
 * 图片编辑 — 左侧画布视口，右侧操作面板（对齐取色器布局）
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { CircleX, Image, ImagePlus, Link2, Loader2, Minus, Plus, RotateCcw, Save, Scan, Scissors, Undo2, Unlink, X } from '@lucide/vue'
import VoidButton from '@/components/VoidButton.vue'
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
  clampSelectionToCanvas,
  cropCanvas,
  isPointInSelection,
  normalizeSelection,
  resizeSelection,
  translateSelection,
  type ResizeHandle,
  type SelectionRect,
  type SelectionShape,
} from '@/utils/image-editor-crop'
import {
  captureEditState,
  createThumbUrl,
  restoreEditState,
  type ImageDocumentSummary,
  type ImageEditState,
} from '@/utils/image-editor-document'
import {
  ICO_SIZE_OPTIONS,
  isValidIcoSize,
  THUMBNAIL_SIZE_OPTIONS,
  type ExportMode,
  type ExportSettings,
  type ImageCompressionPreset,
  type ImageExportFormat,
  type ImageSizeScalePreset,
  sanitizeExportStem,
} from '@/utils/image-editor-export'
import { fileFromPath, isImageFile, isImagePath } from '@/utils/image-file-load'
import { isSquareCanvas, SQUARE_EXPORT_HINT } from '@/utils/image-editor-project'

const EXPORT_MODE_TABS: { id: ExportMode; label: string }[] = [
  { id: 'general', label: '通用' },
  { id: 'thumbnail', label: '缩略图' },
  { id: 'ico', label: 'ICO' },
]

const props = defineProps<{
  documentId: string
  file: File
  imageName: string
  imageWidth: number
  imageHeight: number
  restoredEditState: ImageEditState | null
  documentHistory: ImageDocumentSummary[]
  activeDocumentId: string
  exportMode: ExportMode
  exportName: string
  exportFormatId: string
  exportFormats: ImageExportFormat[]
  compressionPresetId: string
  compressionPresets: ImageCompressionPreset[]
  sizeScalePresetId: string
  sizeScalePresets: ImageSizeScalePreset[]
  thumbnailSizes: number[]
  icoSizes: number[]
  saving?: boolean
  /** 独立编辑窗口内渲染（非主窗口 overlay） */
  standalone?: boolean
}>()

const emit = defineEmits<{
  'update:exportMode': [mode: ExportMode]
  'update:exportName': [name: string]
  'update:exportFormatId': [id: string]
  'update:compressionPresetId': [id: string]
  'update:sizeScalePresetId': [id: string]
  'update:thumbnailSizes': [sizes: number[]]
  'update:icoSizes': [sizes: number[]]
  export: [payload: { canvas: HTMLCanvasElement; settings: ExportSettings }]
  'state-snapshot': [payload: { documentId: string; editState: ImageEditState; thumbUrl: string }]
  'switch-document': [payload: {
    id: string
    snapshot: { documentId: string; editState: ImageEditState; thumbUrl: string }
  }]
  'add-image': [payload: {
    file: File
    snapshot: { documentId: string; editState: ImageEditState; thumbUrl: string }
  }]
  'remove-document': [payload: {
    id: string
    snapshot: { documentId: string; editState: ImageEditState; thumbUrl: string }
  }]
  save: [payload: {
    snapshot: { documentId: string; editState: ImageEditState; thumbUrl: string }
  }]
  'exit-request': [payload: {
    snapshot: { documentId: string; editState: ImageEditState; thumbUrl: string } | null
  }]
  exit: []
}>()

const uploadInputRef = ref<HTMLInputElement | null>(null)
const uploadDragOver = ref(false)

const isTauri = (): boolean =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

let unlistenDragDrop: UnlistenFn | undefined

const SHAPE_OPTIONS: { id: SelectionShape; label: string }[] = [
  { id: 'square', label: '正方形' },
  { id: 'rect', label: '矩形' },
  { id: 'circle', label: '圆形' },
  { id: 'ellipse', label: '椭圆' },
]

const RESIZE_HANDLES: { id: ResizeHandle; cursor: string }[] = [
  { id: 'nw', cursor: 'nwse-resize' },
  { id: 'n', cursor: 'ns-resize' },
  { id: 'ne', cursor: 'nesw-resize' },
  { id: 'w', cursor: 'ew-resize' },
  { id: 'e', cursor: 'ew-resize' },
  { id: 'sw', cursor: 'nesw-resize' },
  { id: 's', cursor: 'ns-resize' },
  { id: 'se', cursor: 'nwse-resize' },
]

const viewportRef = ref<HTMLElement | null>(null)
const originalCanvasRef = ref<HTMLCanvasElement | null>(null)
const workingCanvasRef = ref<HTMLCanvasElement | null>(null)
const viewCanvasRef = ref<HTMLCanvasElement | null>(null)

const ready = ref(false)
const shape = ref<SelectionShape>('square')
const selection = ref<SelectionRect | null>(null)
const draftSelection = ref<SelectionRect | null>(null)
const cropCount = ref(0)
const lockAspectRatio = ref(true)
const exportWidth = ref(0)
const exportHeight = ref(0)
const transform = ref<CanvasTransform>({ panX: 0, panY: 0, scale: 1 })
const baseScale = ref(1)
const isPanning = ref(false)
const isSelecting = ref(false)
const isMovingSelection = ref(false)
const isResizingSelection = ref(false)
const resizeHandle = ref<ResizeHandle | null>(null)

const zoomInputValue = ref('100%')
const zoomInputFocused = ref(false)

let viewCtx: CanvasRenderingContext2D | null = null
let panStartX = 0
let panStartY = 0
let panOriginX = 0
let panOriginY = 0
let selectStart: { x: number; y: number } | null = null
let moveSelectionOrigin: SelectionRect | null = null
let resizeSelectionOrigin: SelectionRect | null = null
let movePointerStart: { x: number; y: number } | null = null
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

const selectionInteractive = computed(() => !!selection.value && !draftSelection.value)

const canvasPanStyle = computed(() => ({
  transform: `translate(${transform.value.panX}px, ${transform.value.panY}px)`,
}))

const viewportCursor = computed(() => {
  if (isPanning.value || isMovingSelection.value || isResizingSelection.value) return 'grabbing'
  if (selectionInteractive.value) return 'default'
  if (selection.value) return 'default'
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

const canvasAspectRatio = computed(() => {
  const canvas = workingCanvasRef.value
  if (!canvas?.width || !canvas.height) return 1
  return canvas.width / canvas.height
})

const exportSizeLabel = computed(() => `${exportWidth.value} × ${exportHeight.value}`)

const thumbnailFormatOptions = computed(() =>
  props.exportFormats.filter((format) => format.id !== 'ico'),
)

const canApplyCrop = computed(() => !!activeSelection.value)

const hasSquareExportSource = computed(() => {
  const working = workingCanvasRef.value
  if (!working) return false
  if (isSquareCanvas(working.width, working.height)) return true

  const sel = selection.value
  if (!sel) return false
  return isSquareCanvas(sel.width, sel.height)
})

const squareExportRequired = computed(() =>
  props.exportMode === 'thumbnail' || props.exportMode === 'ico',
)

const canExport = computed(() => {
  if (!ready.value || !props.exportName.trim()) return false
  if (props.exportMode === 'thumbnail') {
    return props.thumbnailSizes.length > 0 && hasSquareExportSource.value
  }
  if (props.exportMode === 'ico') {
    return props.icoSizes.length > 0 && hasSquareExportSource.value
  }
  return true
})

const exportHint = computed(() => {
  if (!squareExportRequired.value || hasSquareExportSource.value) return ''
  return SQUARE_EXPORT_HINT
})

const resolveSquareExportCanvas = (): HTMLCanvasElement | null => {
  const working = workingCanvasRef.value
  if (!working) return null
  if (isSquareCanvas(working.width, working.height)) return working

  const sel = selection.value
  if (!sel || !isSquareCanvas(sel.width, sel.height)) return null
  return cropCanvas(working, sel, shape.value)
}

const toggleThumbnailSize = (size: number, checked: boolean): void => {
  const next = checked
    ? [...props.thumbnailSizes, size]
    : props.thumbnailSizes.filter((item) => item !== size)
  emit('update:thumbnailSizes', [...next].sort((a, b) => a - b))
}

const toggleIcoSize = (size: number, checked: boolean): void => {
  const next = checked
    ? [...props.icoSizes, size]
    : props.icoSizes.filter((item) => item !== size)
  emit('update:icoSizes', [...next].sort((a, b) => a - b))
}

const getViewportRect = (): DOMRect | null => viewportRef.value?.getBoundingClientRect() ?? null

const syncZoomInput = (): void => {
  if (!zoomInputFocused.value) {
    zoomInputValue.value = formatDisplayZoomPercent(transform.value.scale, baseScale.value)
  }
}

const getWorkingCanvas = (): HTMLCanvasElement | null => workingCanvasRef.value

const syncExportSizeFromCanvas = (): void => {
  const canvas = getWorkingCanvas()
  if (!canvas?.width || !canvas.height) return

  const preset = props.sizeScalePresets.find((p) => p.id === props.sizeScalePresetId)
    ?? props.sizeScalePresets[0]
  exportWidth.value = Math.max(1, Math.round(canvas.width * preset.scale))
  exportHeight.value = Math.max(1, Math.round(canvas.height * preset.scale))
}

const onSizeScalePresetChange = (event: Event): void => {
  const id = (event.target as HTMLSelectElement).value
  emit('update:sizeScalePresetId', id)
  syncExportSizeFromCanvas()
}

const onExportWidthInput = (): void => {
  exportWidth.value = Math.max(1, Math.round(exportWidth.value) || 1)
  if (lockAspectRatio.value) {
    exportHeight.value = Math.max(1, Math.round(exportWidth.value / canvasAspectRatio.value))
  }
}

const onExportHeightInput = (): void => {
  exportHeight.value = Math.max(1, Math.round(exportHeight.value) || 1)
  if (lockAspectRatio.value) {
    exportWidth.value = Math.max(1, Math.round(exportHeight.value * canvasAspectRatio.value))
  }
}

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

const getCanvasPoint = (clientX: number, clientY: number): { x: number; y: number } | null => {
  const working = getWorkingCanvas()
  const rect = getViewportRect()
  if (!working || !rect) return null
  return clientToCanvasPixel(
    clientX,
    clientY,
    rect,
    transform.value,
    working.width,
    working.height,
  )
}

const startMoveSelection = (clientX: number, clientY: number): void => {
  const point = getCanvasPoint(clientX, clientY)
  const sel = selection.value
  if (!point || !sel) return

  isMovingSelection.value = true
  moveSelectionOrigin = { ...sel }
  movePointerStart = point
}

const updateSelectionMove = (clientX: number, clientY: number): void => {
  const working = getWorkingCanvas()
  const origin = moveSelectionOrigin
  const start = movePointerStart
  if (!working || !origin || !start || !isMovingSelection.value) return

  const point = getCanvasPoint(clientX, clientY)
  if (!point) return

  selection.value = translateSelection(
    origin,
    point.x - start.x,
    point.y - start.y,
    working.width,
    working.height,
  )
}

const endMoveSelection = (): void => {
  isMovingSelection.value = false
  moveSelectionOrigin = null
  movePointerStart = null
}

const startResizeSelection = (handle: ResizeHandle, clientX: number, clientY: number): void => {
  const point = getCanvasPoint(clientX, clientY)
  const sel = selection.value
  if (!point || !sel) return

  isResizingSelection.value = true
  resizeHandle.value = handle
  resizeSelectionOrigin = { ...sel }
  movePointerStart = point
}

const updateSelectionResize = (clientX: number, clientY: number): void => {
  const working = getWorkingCanvas()
  const origin = resizeSelectionOrigin
  const start = movePointerStart
  const handle = resizeHandle.value
  if (!working || !origin || !start || !handle || !isResizingSelection.value) return

  const point = getCanvasPoint(clientX, clientY)
  if (!point) return

  const resized = resizeSelection(
    origin,
    handle,
    point.x - start.x,
    point.y - start.y,
    shape.value,
    working.width,
    working.height,
  )
  if (resized) {
    selection.value = resized
  }
}

const endResizeSelection = (): void => {
  isResizingSelection.value = false
  resizeHandle.value = null
  resizeSelectionOrigin = null
  movePointerStart = null
}

const onOverlayBodyPointerDown = (event: PointerEvent): void => {
  if (event.button !== 0 || !selectionInteractive.value) return
  event.preventDefault()
  viewportRef.value?.setPointerCapture(event.pointerId)
  leftPointerDown = true
  pointerDownX = event.clientX
  pointerDownY = event.clientY
  startMoveSelection(event.clientX, event.clientY)
}

const onResizeHandlePointerDown = (handle: ResizeHandle, event: PointerEvent): void => {
  if (event.button !== 0 || !selectionInteractive.value) return
  event.preventDefault()
  viewportRef.value?.setPointerCapture(event.pointerId)
  leftPointerDown = true
  pointerDownX = event.clientX
  pointerDownY = event.clientY
  startResizeSelection(handle, event.clientX, event.clientY)
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

    const point = getCanvasPoint(event.clientX, event.clientY)
    const sel = selection.value
    if (point && sel && isPointInSelection(point, sel)) {
      startMoveSelection(event.clientX, event.clientY)
      return
    }

    startSelect(event.clientX, event.clientY)
  }
}

const onPointerMove = (event: PointerEvent): void => {
  if (isPanning.value) {
    movePan(event.clientX, event.clientY)
    return
  }
  if (isResizingSelection.value) {
    updateSelectionResize(event.clientX, event.clientY)
    return
  }
  if (isMovingSelection.value) {
    updateSelectionMove(event.clientX, event.clientY)
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

  if (isResizingSelection.value) {
    viewportRef.value?.releasePointerCapture(event.pointerId)
    leftPointerDown = false
    endResizeSelection()
    return
  }

  if (isMovingSelection.value) {
    viewportRef.value?.releasePointerCapture(event.pointerId)
    leftPointerDown = false
    endMoveSelection()
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
  syncExportSizeFromCanvas()
  fitToView()
  void emitStateSnapshot()
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
  syncExportSizeFromCanvas()
  fitToView()
  void emitStateSnapshot()
}

const clearSelection = (): void => {
  selection.value = null
  draftSelection.value = null
}

const buildStateSnapshot = async (): Promise<{
  documentId: string
  editState: ImageEditState
  thumbUrl: string
} | null> => {
  const original = originalCanvasRef.value
  const working = workingCanvasRef.value
  if (!original || !working || !ready.value) return null

  const editState = await captureEditState(
    original,
    working,
    cropCount.value,
    shape.value,
    selection.value,
  )
  const thumbUrl = await createThumbUrl(working)
  return { documentId: props.documentId, editState, thumbUrl }
}

const emitStateSnapshot = async (): Promise<void> => {
  const snapshot = await buildStateSnapshot()
  if (!snapshot) return
  emit('state-snapshot', snapshot)
}

const requestSwitchDocument = async (id: string): Promise<void> => {
  if (id === props.activeDocumentId) return
  const snapshot = await buildStateSnapshot()
  if (!snapshot) return
  emit('switch-document', { id, snapshot })
}

const requestRemoveDocument = async (id: string): Promise<void> => {
  if (props.documentHistory.length <= 1) return
  const snapshot = await buildStateSnapshot()
  if (!snapshot) return
  emit('remove-document', { id, snapshot })
}

const resetExportName = (): void => {
  emit('update:exportName', sanitizeExportStem(props.imageName))
}

const processUploadFile = async (file: File | undefined | null): Promise<void> => {
  if (!file || !isImageFile(file)) return

  const snapshot = await buildStateSnapshot()
  if (!snapshot) return
  emit('add-image', { file, snapshot })
}

const onUploadDragOver = (event: DragEvent): void => {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy'
  }
  uploadDragOver.value = true
}

const onUploadDragLeave = (): void => {
  uploadDragOver.value = false
}

const onUploadDrop = async (event: DragEvent): Promise<void> => {
  event.preventDefault()
  uploadDragOver.value = false
  if (isTauri()) return
  await processUploadFile(event.dataTransfer?.files?.[0])
}

const onUploadInputChange = async (event: Event): Promise<void> => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return

  await processUploadFile(file)
}

const requestSave = async (): Promise<void> => {
  const snapshot = await buildStateSnapshot()
  if (!snapshot) return
  emit('save', { snapshot })
}

const requestExit = async (): Promise<void> => {
  const snapshot = await buildStateSnapshot()
  emit('exit-request', { snapshot })
}

const requestExport = (): void => {
  const working = workingCanvasRef.value
  if (!working || !canExport.value) return

  let canvas = working
  if (squareExportRequired.value) {
    const squareCanvas = resolveSquareExportCanvas()
    if (!squareCanvas) return
    canvas = squareCanvas
  }

  emit('export', {
    canvas,
    settings: {
      mode: props.exportMode,
      exportName: props.exportName.trim(),
      formatId: props.exportFormatId,
      compressionPresetId: props.compressionPresetId,
      exportSize: {
        width: exportWidth.value,
        height: exportHeight.value,
      },
      thumbnailSizes: [...props.thumbnailSizes],
      icoSizes: [...props.icoSizes],
    },
  })
}

const loadImage = async (): Promise<void> => {
  await nextTick()
  const original = originalCanvasRef.value
  const working = workingCanvasRef.value
  if (!original || !working) return

  ready.value = false
  try {
    if (props.restoredEditState) {
      await restoreEditState(original, working, props.restoredEditState)
      cropCount.value = props.restoredEditState.cropCount
      shape.value = props.restoredEditState.selectionShape ?? 'square'
      selection.value = clampSelectionToCanvas(
        props.restoredEditState.selection,
        working.width,
        working.height,
      )
    } else {
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
      cropCount.value = 0
      shape.value = 'square'
      selection.value = null
    }

    ready.value = true
    draftSelection.value = null
    lastRenderedScale = -1
    syncExportSizeFromCanvas()
    fitToView()
    void emitStateSnapshot()
  } catch {
    ready.value = false
  }
}

watch(
  () => props.exportMode,
  (mode) => {
    if (mode !== 'ico') return
    const filtered = props.icoSizes.filter(isValidIcoSize)
    if (filtered.length !== props.icoSizes.length) {
      emit('update:icoSizes', filtered)
    }
  },
)

watch(
  () => props.sizeScalePresetId,
  () => syncExportSizeFromCanvas(),
)

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
  const viewport = viewportRef.value
  if (viewport) {
    resizeObserver = new ResizeObserver(() => redrawCanvas())
    resizeObserver.observe(viewport)
  }

  if (props.standalone && isTauri()) {
    try {
      unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === 'over' || event.payload.type === 'enter') {
          uploadDragOver.value = true
          return
        }
        if (event.payload.type === 'leave') {
          uploadDragOver.value = false
          return
        }
        if (event.payload.type === 'drop') {
          const path = event.payload.paths.find(isImagePath)
          uploadDragOver.value = false
          if (!path) return
          void fileFromPath(path).then(processUploadFile)
        }
      })
    } catch {
      // 非 Tauri 环境
    }
  }

  await loadImage()
})

onUnmounted(() => {
  void unlistenDragDrop?.()
  unlistenDragDrop = undefined
  resizeObserver?.disconnect()
  resizeObserver = null
  void buildStateSnapshot().then((snapshot) => {
    if (snapshot) emit('state-snapshot', snapshot)
  })
})
</script>

<template>
  <div class="editor-session" :class="{ 'editor-session--standalone': standalone }">
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
          :class="[
            overlayShapeClass,
            { 'editor-session__overlay-shape--interactive': selectionInteractive },
          ]"
          @pointerdown.stop="onOverlayBodyPointerDown"
        />
        <button
          v-for="handle in RESIZE_HANDLES"
          v-show="selectionInteractive"
          :key="handle.id"
          type="button"
          class="editor-session__resize-handle"
          :class="`editor-session__resize-handle--${handle.id}`"
          :style="{ cursor: handle.cursor }"
          :aria-label="`调整选区${handle.id}`"
          @pointerdown.stop="onResizeHandlePointerDown(handle.id, $event)"
        />
      </div>
      <div v-if="!ready" class="editor-session__loading">
        <Loader2 :size="32" :stroke-width="2" class="editor-session__loading-spin" />
        <p>正在加载图片…</p>
      </div>
    </div>

    <aside class="editor-session__panel">
      <header class="editor-session__panel-head">
        <div class="editor-session__title-row">
          <Image :size="18" :stroke-width="2" class="editor-session__title-icon" />
          <h3 class="editor-session__panel-title">图片编辑</h3>
        </div>
        <p class="editor-session__meta" :title="imageName">{{ imageName }}</p>
        <button
          type="button"
          class="editor-session__upload-btn"
          :class="{ 'editor-session__upload-btn--drag-over': uploadDragOver }"
          @click="uploadInputRef?.click()"
          @dragover="onUploadDragOver"
          @dragleave="onUploadDragLeave"
          @drop="onUploadDrop"
        >
          <ImagePlus :size="14" :stroke-width="2" />
          点击或拖拽上传图片
        </button>
        <input
          ref="uploadInputRef"
          type="file"
          accept="image/*"
          class="editor-session__upload-input"
          @change="onUploadInputChange"
        />
        <div v-if="documentHistory.length > 0" class="editor-session__history">
          <div
            v-for="doc in documentHistory"
            :key="doc.id"
            class="editor-session__history-item"
            :class="{ 'editor-session__history-item--active': doc.id === activeDocumentId }"
          >
            <button
              type="button"
              class="editor-session__history-thumb-btn"
              :title="doc.name"
              @click="requestSwitchDocument(doc.id)"
            >
              <img
                v-if="doc.thumbUrl"
                :src="doc.thumbUrl"
                :alt="doc.name"
                class="editor-session__history-thumb"
              />
              <span v-else class="editor-session__history-fallback">{{ doc.name.slice(0, 1) }}</span>
            </button>
            <button
              v-if="documentHistory.length > 1"
              type="button"
              class="editor-session__history-remove"
              aria-label="删除图片"
              @click.stop="requestRemoveDocument(doc.id)"
            >
              <X :size="12" :stroke-width="2.5" />
            </button>
          </div>
        </div>
      </header>

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

      <div class="editor-session__zoom">
        <span class="editor-session__section-label">视图缩放</span>
        <div class="editor-session__zoom-controls">
          <VoidButton kind="icon" size="medium" title="缩小 (-)" @click="zoomBy(1 / ZOOM_STEP_FACTOR)">
            <Minus :size="14" :stroke-width="2" />
          </VoidButton>
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
          <VoidButton kind="icon" size="medium" title="放大 (+)" @click="zoomBy(ZOOM_STEP_FACTOR)">
            <Plus :size="14" :stroke-width="2" />
          </VoidButton>
          <VoidButton kind="icon" size="medium" title="适应窗口 (0)" @click="fitToView">
            <Scan :size="14" :stroke-width="2" />
          </VoidButton>
        </div>
      </div>

      <section class="editor-session__export-settings">
        <span class="editor-session__section-label">导出设置</span>

        <div class="editor-session__tabs">
          <button
            v-for="tab in EXPORT_MODE_TABS"
            :key="tab.id"
            type="button"
            class="editor-session__tab"
            :class="{ 'editor-session__tab--active': exportMode === tab.id }"
            @click="emit('update:exportMode', tab.id)"
          >
            {{ tab.label }}
          </button>
        </div>

        <label class="editor-session__field">
          <span>导出名称</span>
          <div class="editor-session__name-row">
            <input
              :value="exportName"
              type="text"
              class="editor-session__text-input editor-session__text-input--flex"
              placeholder="文件名"
              @input="emit('update:exportName', ($event.target as HTMLInputElement).value)"
            />
            <VoidButton
              kind="icon" size="medium"
              title="恢复文件名"
              aria-label="恢复文件名"
              @click="resetExportName"
            >
              <Undo2 :size="14" :stroke-width="2" />
            </VoidButton>
          </div>
        </label>

        <template v-if="exportMode === 'general'">
          <div class="editor-session__subsection">
            <span class="editor-session__subsection-label">图片尺寸</span>
            <label class="editor-session__field">
              <span>缩放比例</span>
              <select
                :value="sizeScalePresetId"
                class="editor-session__select void-select"
                @change="onSizeScalePresetChange"
              >
                <option v-for="preset in sizeScalePresets" :key="preset.id" :value="preset.id">
                  {{ preset.label }}
                </option>
              </select>
            </label>
            <div class="editor-session__size-row">
              <label class="editor-session__size-field">
                <span>宽</span>
                <input
                  v-model.number="exportWidth"
                  type="number"
                  min="1"
                  class="editor-session__size-input"
                  @input="onExportWidthInput"
                />
              </label>
              <VoidButton
                kind="icon" size="medium"
                :active="lockAspectRatio"
                :title="lockAspectRatio ? '已锁定等比例' : '自由比例'"
                :aria-label="lockAspectRatio ? '已锁定等比例' : '自由比例'"
                @click="lockAspectRatio = !lockAspectRatio"
              >
                <Link2 v-if="lockAspectRatio" :size="14" :stroke-width="2" />
                <Unlink v-else :size="14" :stroke-width="2" />
              </VoidButton>
              <label class="editor-session__size-field">
                <span>高</span>
                <input
                  v-model.number="exportHeight"
                  type="number"
                  min="1"
                  class="editor-session__size-input"
                  @input="onExportHeightInput"
                />
              </label>
            </div>
            <span class="editor-session__info-sub">
              画布 {{ workingSizeLabel }} · 导出 {{ exportSizeLabel }}
            </span>
          </div>

          <div class="editor-session__subsection">
            <span class="editor-session__subsection-label">导出要求</span>
            <div class="editor-session__export-row">
              <label class="editor-session__field editor-session__field--half">
                <span>格式</span>
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
              <label class="editor-session__field editor-session__field--half">
                <span>压缩</span>
                <select
                  :value="compressionPresetId"
                  class="editor-session__select void-select"
                  @change="emit('update:compressionPresetId', ($event.target as HTMLSelectElement).value)"
                >
                  <option v-for="preset in compressionPresets" :key="preset.id" :value="preset.id">
                    {{ preset.label }}
                  </option>
                </select>
              </label>
            </div>
          </div>
        </template>

        <div v-else-if="exportMode === 'thumbnail'" class="editor-session__subsection">
          <span class="editor-session__subsection-label">缩略图尺寸（可多选）</span>
          <div class="editor-session__size-chips">
            <label
              v-for="size in THUMBNAIL_SIZE_OPTIONS"
              :key="size"
              class="editor-session__size-chip"
              :class="{ 'editor-session__size-chip--active': thumbnailSizes.includes(size) }"
            >
              <input
                type="checkbox"
                class="editor-session__size-chip-input"
                :checked="thumbnailSizes.includes(size)"
                @change="toggleThumbnailSize(size, ($event.target as HTMLInputElement).checked)"
              />
              <span>{{ size }}×{{ size }}</span>
            </label>
          </div>
          <div class="editor-session__export-row">
            <label class="editor-session__field editor-session__field--half">
              <span>格式</span>
              <select
                :value="exportFormatId"
                class="editor-session__select void-select"
                @change="emit('update:exportFormatId', ($event.target as HTMLSelectElement).value)"
              >
                <option v-for="fmt in thumbnailFormatOptions" :key="fmt.id" :value="fmt.id">
                  {{ fmt.label }}
                </option>
              </select>
            </label>
            <label class="editor-session__field editor-session__field--half">
              <span>压缩</span>
              <select
                :value="compressionPresetId"
                class="editor-session__select void-select"
                @change="emit('update:compressionPresetId', ($event.target as HTMLSelectElement).value)"
              >
                <option v-for="preset in compressionPresets" :key="preset.id" :value="preset.id">
                  {{ preset.label }}
                </option>
              </select>
            </label>
          </div>
          <span class="editor-session__info-sub">命名：名称_尺寸.后缀 · 导出至所选目录</span>
          <p v-if="exportHint" class="editor-session__warn">{{ exportHint }}</p>
        </div>

        <div v-else class="editor-session__subsection">
          <span class="editor-session__subsection-label">ICO 尺寸（可多选，最大 256）</span>
          <div class="editor-session__size-chips">
            <label
              v-for="size in ICO_SIZE_OPTIONS"
              :key="size"
              class="editor-session__size-chip"
              :class="{ 'editor-session__size-chip--active': icoSizes.includes(size) }"
            >
              <input
                type="checkbox"
                class="editor-session__size-chip-input"
                :checked="icoSizes.includes(size)"
                @change="toggleIcoSize(size, ($event.target as HTMLInputElement).checked)"
              />
              <span>{{ size }}×{{ size }}</span>
            </label>
          </div>
          <span class="editor-session__info-sub">命名：名称_尺寸.ico · 导出至所选目录</span>
          <p v-if="exportHint" class="editor-session__warn">{{ exportHint }}</p>
        </div>
      </section>

      <div class="editor-session__toolbar">
        <VoidButton
          kind="icon"
          size="large"
          title="确认裁剪"
          :disabled="!canApplyCrop"
          @click="applyCrop"
        >
          <Scissors :size="16" :stroke-width="2" />
        </VoidButton>
        <VoidButton
          kind="icon"
          size="large"
          title="恢复"
          :disabled="cropCount === 0"
          @click="resetToOriginal"
        >
          <RotateCcw :size="16" :stroke-width="2" />
        </VoidButton>
        <VoidButton
          kind="icon"
          size="large"
          title="清除选区"
          :disabled="!activeSelection"
          @click="clearSelection"
        >
          <CircleX :size="16" :stroke-width="2" />
        </VoidButton>
        <VoidButton
          kind="button"
          size="large"
          variant="primary"
          grow
          :disabled="!canExport"
          @click="requestExport"
        >
          导出
        </VoidButton>
      </div>

      <div class="editor-session__actions">
        <VoidButton
          variant="accent"
          grow
          size="large"
          :disabled="saving || !ready"
          :loading="saving"
          @click="requestSave"
        >
          <Loader2 v-if="saving" :size="16" :stroke-width="2" class="editor-session__btn-spin" />
          <Save v-else :size="16" :stroke-width="2" />
          {{ saving ? '保存中…' : '保存' }}
        </VoidButton>
        <VoidButton variant="secondary" grow size="large" @click="requestExit">
          <X :size="16" :stroke-width="2" />
          退出编辑
        </VoidButton>
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

  &--standalone {
    position: relative;
    inset: auto;
    z-index: auto;
    width: 100%;
    height: 100%;
    min-height: 0;
  }

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
    pointer-events: none;

    &--interactive {
      pointer-events: auto;
      cursor: grab;

      &:active {
        cursor: grabbing;
      }
    }

    &--circle {
      border-radius: 50%;
    }

    &--ellipse {
      border-radius: 50%;
    }
  }

  &__resize-handle {
    position: absolute;
    width: 10px;
    height: 10px;
    margin: 0;
    padding: 0;
    border: 2px solid #f3e8ff;
    border-radius: 2px;
    background: #c084fc;
    box-sizing: border-box;
    pointer-events: auto;
    transform: translate(-50%, -50%);

    &--nw { left: 0; top: 0; }
    &--n { left: 50%; top: 0; }
    &--ne { left: 100%; top: 0; }
    &--w { left: 0; top: 50%; }
    &--e { left: 100%; top: 50%; }
    &--sw { left: 0; top: 100%; }
    &--s { left: 50%; top: 100%; }
    &--se { left: 100%; top: 100%; }
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
    gap: 10px;
    background: rgba(22, 23, 29, 0.98);
    border-left: 1px solid rgba(255, 255, 255, 0.08);
    overflow: hidden;
    min-height: 0;
  }

  &__panel-head {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 0 0 auto;
    min-width: 0;
  }

  &__title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  &__title-icon {
    flex-shrink: 0;
    color: #c084fc;
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
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  &__upload-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 100%;
    height: 32px;
    margin-top: 2px;
    border-radius: 6px;
    border: 1px dashed rgba(192, 132, 252, 0.35);
    background: rgba(192, 132, 252, 0.08);
    color: #e9d5ff;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;

    &:hover {
      background: rgba(192, 132, 252, 0.14);
    }

    &--drag-over {
      border-color: rgba(192, 132, 252, 0.75);
      background: rgba(192, 132, 252, 0.2);
      color: #f3e8ff;
    }
  }

  &__upload-input {
    display: none;
  }

  &__history {
    display: flex;
    gap: 6px;
    overflow-x: auto;
    padding-bottom: 2px;
    margin-top: 4px;
    scrollbar-width: none;
    -ms-overflow-style: none;

    &::-webkit-scrollbar {
      display: none;
    }
  }

  &__history-item {
    position: relative;
    flex: 0 0 auto;
  }

  &__history-thumb-btn {
    width: 44px;
    height: 44px;
    padding: 0;
    border-radius: 6px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.04);
    overflow: hidden;
    cursor: pointer;
  }

  &__history-item--active &__history-thumb-btn {
    border-color: rgba(192, 132, 252, 0.7);
    box-shadow: 0 0 0 1px rgba(192, 132, 252, 0.25);
  }

  &__history-remove {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: rgba(15, 15, 20, 0.95);
    color: #fca5a5;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
  }

  &__history-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  &__history-fallback {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    font-size: 14px;
    color: #9ca3af;
  }

  &__section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    flex: 0 0 auto;
  }

  &__section-label {
    font-size: 12px;
    color: #9ca3af;
    font-weight: 600;
  }

  &__export-settings {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 10px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
    overflow-y: auto;
  }

  &__tabs {
    display: flex;
    gap: 4px;
    padding: 3px;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.2);
  }

  &__tab {
    flex: 1;
    min-width: 0;
    padding: 6px 4px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: #9ca3af;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;

    &--active {
      background: rgba(192, 132, 252, 0.2);
      color: #e9d5ff;
    }

    &:hover:not(&--active) {
      color: #e5e7eb;
    }
  }

  &__text-input {
    width: 100%;
    height: 32px;
    padding: 0 8px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 13px;
    box-sizing: border-box;

    &--flex {
      flex: 1;
      min-width: 0;
    }

    &:focus {
      outline: none;
      border-color: rgba(192, 132, 252, 0.5);
    }
  }

  &__name-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  &__size-chips {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
  }

  &__size-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
    color: #9ca3af;
    font-size: 11px;
    cursor: pointer;
    user-select: none;

    &--active {
      border-color: rgba(192, 132, 252, 0.45);
      background: rgba(192, 132, 252, 0.12);
      color: #e9d5ff;
    }
  }

  &__size-chip-input {
    width: 12px;
    height: 12px;
    accent-color: #c084fc;
    cursor: pointer;
  }

  &__subsection {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  &__subsection-label {
    font-size: 11px;
    color: #6b7280;
  }

  &__field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 11px;
    color: #9ca3af;
    min-width: 0;

    &--half {
      flex: 1;
      min-width: 0;
    }
  }

  &__export-row {
    display: flex;
    gap: 8px;
    min-width: 0;
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

  &__info-sub {
    font-size: 11px;
    color: #6b7280;
  }

  &__zoom {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 0 0 auto;
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

  &__size-row {
    display: flex;
    align-items: flex-end;
    gap: 6px;
  }

  &__size-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-width: 0;
    font-size: 11px;
    color: #9ca3af;
  }

  &__size-input {
    width: 100%;
    height: 32px;
    padding: 0 8px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 13px;
    font-family: ui-monospace, 'Cascadia Code', monospace;
    box-sizing: border-box;

    &:focus {
      outline: none;
      border-color: rgba(192, 132, 252, 0.5);
    }
  }

  &__select {
    width: 100%;
    padding: 8px 10px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 13px;
    box-sizing: border-box;
  }

  &__toolbar {
    display: flex;
    align-items: stretch;
    gap: 6px;
    flex: 0 0 auto;
  }

  &__actions {
    flex: 0 0 auto;
    margin-top: auto;
    padding-top: 4px;
    display: flex;
    gap: 8px;
  }

  &__btn-spin {
    animation: spin 0.9s linear infinite;
  }

  &__warn {
    margin: 6px 0 0;
    font-size: 11px;
    line-height: 1.45;
    color: #fcd34d;
  }
}

@keyframes editor-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
