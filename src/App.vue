<script setup lang="ts">
/**
 * App.vue
 * Void 主窗口根组件
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Settings } from '@lucide/vue'
import { useRouter } from 'vue-router'
import { initWindow } from '@/api/clash-service'
import AboutDialog from '@/components/AboutDialog.vue'
import SettingsDialog from '@/components/SettingsDialog.vue'

const router = useRouter()
const aboutVisible = ref(false)
const settingsVisible = ref(false)
const isMainWindow = ref(false)

const unlisteners: UnlistenFn[] = []

onMounted(async () => {
  try {
    const appWindow = getCurrentWindow()
    const label = appWindow.label
    isMainWindow.value = label === 'main'
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
    <header v-if="isMainWindow" class="void-window__chrome">
      <button
        type="button"
        class="void-window__settings"
        aria-label="设置"
        @click="settingsVisible = true"
      >
        <Settings :size="16" :stroke-width="2" />
      </button>
    </header>
    <router-view class="void-window__content" />
    <SettingsDialog
      :visible="settingsVisible"
      @close="settingsVisible = false"
      @show-about="aboutVisible = true"
    />
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

  &__chrome {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-height: 36px;
    padding: 4px clamp(12px, 3vw, 20px) 0;
  }

  &__settings {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--void-text-dim);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;

    &:hover {
      background: rgba(255, 255, 255, 0.08);
      color: var(--void-accent);
    }
  }

  &__content {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
}
</style>
