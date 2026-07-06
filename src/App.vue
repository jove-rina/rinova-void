<script setup lang="ts">
/**
 * App.vue
 * Void 主窗口根组件
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import { initWindow } from '@/api/clash-service'
import AboutDialog from '@/components/AboutDialog.vue'
import { queueColorPickerAutoStart } from '@/utils/color-picker-launch'
import WindowHeader from '@/components/WindowHeader.vue'

const appWindow = getCurrentWindow()
const router = useRouter()
const aboutVisible = ref(false)

const unlisteners: UnlistenFn[] = []

onMounted(async () => {
  try {
    await initWindow()
    unlisteners.push(
      await listen('open-color-picker', () => {
        queueColorPickerAutoStart()
        if (router.currentRoute.value.path !== '/tool/color-picker') {
          void router.push('/tool/color-picker')
        }
      }),
      await listen<string>('open-tool', (event) => {
        if (router.currentRoute.value.path !== event.payload) {
          void router.push(event.payload)
        }
      }),
      await listen('open-about', () => {
        aboutVisible.value = true
      }),
    )
  } catch {
    // 非 Tauri 环境（如 vite 浏览器预览）
  }
})

onUnmounted(() => {
  for (const unlisten of unlisteners) {
    void unlisten()
  }
})

const handleClose = async (): Promise<void> => {
  await appWindow.close()
}

const goHome = (): void => {
  if (router.currentRoute.value.path !== '/') {
    router.push('/')
  }
}
</script>

<template>
  <div class="void-window">
    <WindowHeader @close="handleClose" @dblclick="goHome" />
    <router-view />
    <AboutDialog :visible="aboutVisible" @close="aboutVisible = false" />
  </div>
</template>

<style lang="less" scoped>
.void-window {
  width: 100vw;
  height: 100vh;
  background: var(--void-bg);
  border: 1px solid var(--void-border);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
