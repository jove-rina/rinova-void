/**
 * useColorPicker.ts
 * 取色器 — 截屏快照 + 多色取色 + 记录持久化
 */
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  finishPicker,
  listPickerMonitors,
  MAGNIFY_OPTIONS,
  refreshPicker,
  startPicker,
  type MonitorInfo,
  type StartPickerResult,
} from '@/api/color-picker'
import {
  createColorRecord,
  exportColorRecords,
  findRecordByHex,
  loadColorRecords,
  nextDefaultColorName,
  saveColorRecords,
  trimColorRecords,
  type ColorRecord,
  type ColorRecordExportFormat,
} from '@/utils/color-records'
import { toHsl, toRgb } from '@/utils/color-format'

const TOAST_MS = 2500
const TOAST_ACTION_MS = 8000

export interface ToastAction {
  label: string
  run: () => void | Promise<void>
}

const waitForPaint = (): Promise<void> =>
  new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
  })

export const useColorPicker = () => {
  const startRadius = ref(0)
  const hideAppOnCapture = ref(true)
  const selectedMonitorIndex = ref(0)
  const captureAllScreens = ref(false)
  const monitors = ref<MonitorInfo[]>([])
  const monitorsLoading = ref(false)
  const capturing = ref(false)
  const refreshingCapture = ref(false)
  const session = ref<StartPickerResult | null>(null)
  const errorMsg = ref('')
  const successMsg = ref('')
  const successAction = ref<ToastAction | null>(null)

  const records = ref<ColorRecord[]>(loadColorRecords())
  const sessionPickIds = ref<string[]>([])

  let toastTimer: ReturnType<typeof setTimeout> | undefined

  const persistRecords = (): void => {
    saveColorRecords(records.value)
  }

  watch(records, persistRecords, { deep: true })

  const clearToast = (): void => {
    if (toastTimer) {
      clearTimeout(toastTimer)
      toastTimer = undefined
    }
    successAction.value = null
  }

  const showToast = (
    kind: 'success' | 'error',
    msg: string,
    action?: ToastAction,
  ): void => {
    clearToast()
    if (kind === 'success') {
      successMsg.value = msg
      errorMsg.value = ''
      successAction.value = action ?? null
    } else {
      errorMsg.value = msg
      successMsg.value = ''
      successAction.value = null
    }
    const duration = kind === 'success' && action ? TOAST_ACTION_MS : TOAST_MS
    toastTimer = setTimeout(() => {
      if (kind === 'success') {
        successMsg.value = ''
      } else {
        errorMsg.value = ''
      }
      successAction.value = null
    }, duration)
  }

  const showSuccess = (msg: string, action?: ToastAction): void => {
    showToast('success', msg, action)
  }

  const runSuccessAction = (): void => {
    const action = successAction.value
    if (!action) return
    clearToast()
    successMsg.value = ''
    Promise.resolve(action.run()).catch((e: unknown) => {
      showError(e instanceof Error ? e.message : '操作失败')
    })
  }

  const showError = (msg: string): void => {
    showToast('error', msg)
  }

  const cleanupSession = async (): Promise<void> => {
    session.value = null
    sessionPickIds.value = []
    try {
      await getCurrentWindow().setFullscreen(false)
    } catch {
      // 非 Tauri 环境
    }
  }

  const loadMonitors = async (): Promise<void> => {
    monitorsLoading.value = true
    try {
      const list = await listPickerMonitors()
      monitors.value = list
      const primary = list.find((m) => m.is_primary)
      selectedMonitorIndex.value = primary?.index ?? list[0]?.index ?? 0
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e))
    } finally {
      monitorsLoading.value = false
    }
  }

  const handleStartPick = async (): Promise<void> => {
    errorMsg.value = ''
    successMsg.value = ''
    sessionPickIds.value = []
    capturing.value = true
    try {
      await nextTick()
      await waitForPaint()
      session.value = await startPicker(
        startRadius.value,
        hideAppOnCapture.value,
        selectedMonitorIndex.value,
        captureAllScreens.value,
      )
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e))
    } finally {
      capturing.value = false
    }
  }

  const handleSessionRefresh = async (
    monitorIndex: number,
    captureAll: boolean,
    radius: number,
  ): Promise<void> => {
    if (!session.value) return
    errorMsg.value = ''
    refreshingCapture.value = true
    try {
      await nextTick()
      await waitForPaint()
      session.value = await refreshPicker(
        hideAppOnCapture.value,
        monitorIndex,
        captureAll,
        radius,
      )
      selectedMonitorIndex.value = monitorIndex
      captureAllScreens.value = captureAll
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e))
    } finally {
      refreshingCapture.value = false
    }
  }

  const handleSessionRadiusChange = (radius: number): void => {
    if (!session.value || session.value.radius === radius) return
    session.value = { ...session.value, radius }
  }

  const handleSessionPick = (rgb: [number, number, number]): void => {
    const record = createColorRecord(rgb, records.value)
    const existing = findRecordByHex(records.value, record.hex)
    if (existing) {
      showSuccess(`颜色已存在：${existing.name}`)
      return
    }
    records.value = trimColorRecords([record, ...records.value])
    sessionPickIds.value = [record.id, ...sessionPickIds.value]
    showSuccess(`已记录 ${record.hex}`)
  }

  const handleRemoveRecord = (id: string): void => {
    records.value = records.value.filter((record) => record.id !== id)
    sessionPickIds.value = sessionPickIds.value.filter((pickId) => pickId !== id)
  }

  const handleRenameRecord = (id: string, name: string): void => {
    const trimmed = name.trim()
    records.value = records.value.map((record) => {
      if (record.id !== id) return record
      return {
        ...record,
        name: trimmed || nextDefaultColorName(records.value.filter((item) => item.id !== id)),
      }
    })
  }

  const copyText = async (text: string, successLabel: string): Promise<void> => {
    try {
      await navigator.clipboard.writeText(text)
      showSuccess(`已复制 ${successLabel}`)
    } catch (e) {
      showError(e instanceof Error ? e.message : '复制失败')
    }
  }

  const handleCopyRecordHex = async (record: ColorRecord): Promise<void> => {
    await copyText(record.hex, record.hex)
  }

  const handleCopyRecordRgb = async (record: ColorRecord): Promise<void> => {
    const rgb = toRgb(record.r, record.g, record.b)
    await copyText(rgb, rgb)
  }

  const handleCopyRecordHsl = async (record: ColorRecord): Promise<void> => {
    const hsl = toHsl(record.r, record.g, record.b)
    await copyText(hsl, hsl)
  }

  const handleExportRecords = async (format: ColorRecordExportFormat): Promise<void> => {
    if (records.value.length === 0) {
      showError('没有可导出的记录')
      return
    }
    try {
      const path = await exportColorRecords(records.value, format)
      const label = format === 'json' ? 'JSON' : format === 'markdown' ? 'Markdown' : 'CSV'
      const count = records.value.length
      if (path) {
        showSuccess(`已导出 ${count} 条记录（${label}）`, {
          label: '打开目录',
          run: async () => {
            const { revealExportPath } = await import('@/api/export')
            await revealExportPath(path)
          },
        })
      } else {
        showSuccess(`已导出 ${count} 条记录（${label}）`)
      }
    } catch (e) {
      showError(e instanceof Error ? e.message : '导出失败')
    }
  }

  const handleExitPick = async (): Promise<void> => {
    const count = sessionPickIds.value.length
    try {
      await finishPicker(true)
      await cleanupSession()
      if (count > 0) {
        showSuccess(`本次共取 ${count} 个颜色`)
      }
    } catch (e) {
      showError(e instanceof Error ? e.message : String(e))
    }
  }

  const handleSnapshotLoadError = (message: string): void => {
    showError(message)
    void handleExitPick()
  }

  onMounted(() => {
    void loadMonitors()
  })

  onUnmounted(() => {
    clearToast()
    if (session.value) {
      void finishPicker(true).finally(() => {
        void cleanupSession()
      })
    }
  })

  return {
    startRadius,
    hideAppOnCapture,
    selectedMonitorIndex,
    captureAllScreens,
    monitors,
    monitorsLoading,
    capturing,
    refreshingCapture,
    session,
    records,
    errorMsg,
    successMsg,
    successAction,
    magnifyOptions: MAGNIFY_OPTIONS,
    handleStartPick,
    handleSessionRefresh,
    handleSessionRadiusChange,
    handleSessionPick,
    handleRemoveRecord,
    handleRenameRecord,
    handleCopyRecordHex,
    handleCopyRecordRgb,
    handleCopyRecordHsl,
    handleExportRecords,
    runSuccessAction,
    handleExitPick,
    handleSnapshotLoadError,
  }
}
