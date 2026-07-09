<script setup lang="ts">
/**
 * App.vue
 * Void 主窗口根组件
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useRouter } from 'vue-router'
import { initWindow } from '@/api/clash-service'
import AboutDialog from '@/components/AboutDialog.vue'

const router = useRouter()
const aboutVisible = ref(false)

const unlisteners: UnlistenFn[] = []

onMounted(async () => {
  try {
    const appWindow = getCurrentWindow()
    const label = appWindow.label
    if (label === 'image-editor') {
      await router.replace('/tool/image-editor/session')
    } else if (label === 'color-picker') {
      await router.replace('/tool/color-picker/session')
    }

    await initWindow()
    unlisteners.push(
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
</script>

<template>
  <div class="void-window">
    <router-view class="void-window__content" />
    <AboutDialog :visible="aboutVisible" @close="aboutVisible = false" />
  </div>
</template>

<style lang="less" scoped>
.void-window {
  width: 100vw;
  height: 100vh;
  background: var(--void-bg);
  overflow: hidden;
  display: flex;
  flex-direction: column;

  &__content {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
}
</style>
