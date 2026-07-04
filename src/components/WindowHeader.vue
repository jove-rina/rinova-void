<script setup lang="ts">
/**
 * WindowHeader.vue
 * 自定义窗口标题栏（无边框 Tauri 窗口专用）
 *
 * - 左侧：应用名，支持拖拽移动窗口；双击向父组件 emit dblclick（回首页）
 * - 右侧：关闭按钮，emit close
 * - data-tauri-drag-region / startDragging：macOS 窗口拖拽 API
 */
import { getCurrentWindow } from '@tauri-apps/api/window'
import { X } from '@lucide/vue'

/** 向父组件（App.vue）上报的用户操作 */
const emit = defineEmits<{
  close: []
  dblclick: []
}>()

const appWindow = getCurrentWindow()

/**
 * 鼠标按下时启动窗口拖拽。
 * 忽略非左键与关闭按钮上的按下，避免误触。
 *
 * @param e - 标题栏区域的 mousedown 事件
 */
const startDrag = async (e: MouseEvent): Promise<void> => {
  if (e.button !== 0) return
  if ((e.target as HTMLElement).closest('.window-header__close')) return
  try {
    await appWindow.startDragging()
  } catch {
    // 非 Tauri 环境
  }
}
</script>

<template>
  <div class="window-header" data-tauri-drag-region @mousedown="startDrag">
    <div
      class="window-header__title"
      data-tauri-drag-region
      @mousedown="startDrag"
      @dblclick.stop="emit('dblclick')"
    >
      Void
    </div>
    <button
      class="window-header__close"
      aria-label="关闭窗口"
      @click="emit('close')"
    >
      <X :size="12" :stroke-width="2" />
    </button>
  </div>
</template>

<style lang="less" scoped>
.window-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding: 0 12px;
  user-select: none;
  -webkit-app-region: drag;
  app-region: drag;

  &__title {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
    color: var(--void-text-dim);
    letter-spacing: 1px;
    text-transform: uppercase;
    -webkit-app-region: drag;
    app-region: drag;
  }

  &__close {
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--void-text-dim);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s, color 0.15s;
    -webkit-app-region: no-drag;
    app-region: no-drag;

    &:hover {
      background: rgba(255, 59, 48, 0.2);
      color: #ff3b30;
    }
  }
}
</style>
