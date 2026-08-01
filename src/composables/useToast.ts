/**
 * useToast.ts
 * 全局 Toast 状态与自动关闭逻辑
 */
import { ref } from 'vue'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface ToastAction {
  label: string
  run: () => void | Promise<void>
}

export interface ToastState {
  type: ToastType
  message: string
  action?: ToastAction
}

const TOAST_MS = 2500
const TOAST_ACTION_MS = 8000
const TOAST_LEAVE_MS = 3000

export const useToast = () => {
  const state = ref<ToastState | null>(null)

  let timer: ReturnType<typeof setTimeout> | undefined

  const clearTimer = (): void => {
    if (timer) {
      clearTimeout(timer)
      timer = undefined
    }
  }

  const dismiss = (): void => {
    clearTimer()
    state.value = null
  }

  const scheduleDismiss = (delayMs: number): void => {
    clearTimer()
    timer = setTimeout(() => {
      timer = undefined
      dismiss()
    }, delayMs)
  }

  const show = (type: ToastType, message: string, action?: ToastAction): void => {
    dismiss()
    state.value = { type, message, action }
    scheduleDismiss(action ? TOAST_ACTION_MS : TOAST_MS)
  }

  const showSuccess = (message: string, action?: ToastAction): void => {
    show('success', message, action)
  }

  const showError = (message: string): void => {
    show('error', message)
  }

  const showWarning = (message: string): void => {
    show('warning', message)
  }

  const showInfo = (message: string, action?: ToastAction): void => {
    show('info', message, action)
  }

  const onMouseEnter = (): void => {
    if (!state.value) return
    clearTimer()
  }

  const onMouseLeave = (): void => {
    if (!state.value) return
    scheduleDismiss(TOAST_LEAVE_MS)
  }

  const runAction = (): void => {
    const action = state.value?.action
    if (!action) return
    dismiss()
    Promise.resolve(action.run()).catch((e: unknown) => {
      showError(e instanceof Error ? e.message : '操作失败')
    })
  }

  return {
    state,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    dismiss,
    onMouseEnter,
    onMouseLeave,
    runAction,
  }
}

export type UseToastReturn = ReturnType<typeof useToast>
