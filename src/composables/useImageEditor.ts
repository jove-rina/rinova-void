/**
 * useImageEditor.ts
 * 图片编辑器 — 上传、会话与导出
 */
import { ref } from 'vue'
import { exportCanvasImage, revealExportPath } from '@/api/image-editor'
import { IMAGE_EXPORT_FORMATS } from '@/utils/image-editor-export'

const TOAST_MS = 2500
const TOAST_ACTION_MS = 8000

export interface LoadedImage {
  file: File
  name: string
  width: number
  height: number
}

export interface ToastAction {
  label: string
  run: () => void | Promise<void>
}

export const useImageEditor = () => {
  const loadedImage = ref<LoadedImage | null>(null)
  const sessionActive = ref(false)
  const loading = ref(false)
  const exportFormatId = ref(IMAGE_EXPORT_FORMATS[0].id)
  const errorMsg = ref('')
  const successMsg = ref('')
  const successAction = ref<ToastAction | null>(null)

  let toastTimer: ReturnType<typeof setTimeout> | undefined
  let activeToastKind: 'success' | 'error' | null = null

  const dismissToast = (): void => {
    if (activeToastKind === 'success') successMsg.value = ''
    else if (activeToastKind === 'error') errorMsg.value = ''
    activeToastKind = null
    successAction.value = null
  }

  const scheduleToastDismiss = (delayMs: number): void => {
    if (toastTimer) clearTimeout(toastTimer)
    toastTimer = setTimeout(() => {
      toastTimer = undefined
      dismissToast()
    }, delayMs)
  }

  const showSuccess = (message: string, action?: ToastAction): void => {
    dismissToast()
    activeToastKind = 'success'
    successMsg.value = message
    successAction.value = action ?? null
    scheduleToastDismiss(action ? TOAST_ACTION_MS : TOAST_MS)
  }

  const showError = (message: string): void => {
    dismissToast()
    activeToastKind = 'error'
    errorMsg.value = message
    scheduleToastDismiss(TOAST_MS)
  }

  const handleToastMouseEnter = (): void => {
    if (toastTimer) {
      clearTimeout(toastTimer)
      toastTimer = undefined
    }
  }

  const handleToastMouseLeave = (): void => {
    if (!successMsg.value && !errorMsg.value) return
    scheduleToastDismiss(successAction.value ? TOAST_ACTION_MS : TOAST_MS)
  }

  const runSuccessAction = (): void => {
    const action = successAction.value
    if (!action) return
    void action.run()
  }

  const readImageMeta = async (file: File): Promise<LoadedImage> => {
    const bitmap = await createImageBitmap(file)
    const meta: LoadedImage = {
      file,
      name: file.name,
      width: bitmap.width,
      height: bitmap.height,
    }
    bitmap.close()
    return meta
  }

  const handleFileSelect = async (file: File | null | undefined): Promise<void> => {
    if (!file || !file.type.startsWith('image/')) {
      showError('请选择有效的图片文件')
      return
    }
    loading.value = true
    try {
      loadedImage.value = await readImageMeta(file)
      sessionActive.value = false
    } catch {
      showError('图片加载失败')
      loadedImage.value = null
    } finally {
      loading.value = false
    }
  }

  const handleStartEdit = (): void => {
    if (!loadedImage.value) return
    sessionActive.value = true
  }

  const handleExitEdit = (): void => {
    sessionActive.value = false
  }

  const handleExportCanvas = async (canvas: HTMLCanvasElement): Promise<void> => {
    if (!loadedImage.value) return
    try {
      const path = await exportCanvasImage(
        canvas,
        loadedImage.value.name,
        exportFormatId.value,
      )
      showSuccess('图片已导出到下载目录', {
        label: '在文件夹中显示',
        run: () => revealExportPath(path),
      })
    } catch (e) {
      showError(e instanceof Error ? e.message : '导出失败')
    }
  }

  return {
    loadedImage,
    sessionActive,
    loading,
    exportFormatId,
    exportFormats: IMAGE_EXPORT_FORMATS,
    errorMsg,
    successMsg,
    successAction,
    handleFileSelect,
    handleStartEdit,
    handleExitEdit,
    handleExportCanvas,
    handleToastMouseEnter,
    handleToastMouseLeave,
    runSuccessAction,
    showSuccess,
    showError,
  }
}
