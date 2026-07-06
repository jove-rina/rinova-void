/**
 * color-picker-launch.ts
 * 托盘等外部入口触发取色时的启动队列
 */
import { ref } from 'vue'

export const pendingColorPickerAutoStart = ref(false)

export const queueColorPickerAutoStart = (): void => {
  pendingColorPickerAutoStart.value = true
}

export const consumeColorPickerAutoStart = (): boolean => {
  if (!pendingColorPickerAutoStart.value) return false
  pendingColorPickerAutoStart.value = false
  return true
}
