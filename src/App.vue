<script setup lang="ts">
/**
 * App.vue
 * Void 主窗口根组件
 *
 * 布局：自定义标题栏（WindowHeader）+ 路由视图（首页 / 工具页）。
 * 挂载时调用 Tauri 窗口初始化；关闭与双击标题栏由本组件编排。
 */
import { onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useRouter } from 'vue-router'
import { initWindow } from '@/api/clash-service'
import WindowHeader from '@/components/WindowHeader.vue'

/** 当前 Tauri 窗口实例，用于 close 等原生操作 */
const appWindow = getCurrentWindow()
const router = useRouter()

/**
 * 应用挂载后初始化窗口样式（圆角、透明背景等）。
 * 浏览器 dev 预览时 invoke 不可用，catch 后静默继续。
 */
onMounted(async () => {
  try {
    await initWindow()
  } catch {
    // 非 Tauri 环境（如 vite 浏览器预览）
  }
})

/**
 * 响应标题栏关闭按钮：销毁当前窗口。
 */
const handleClose = async (): Promise<void> => {
  await appWindow.close()
}

/**
 * 双击标题栏：若不在首页则导航回 `/`。
 * 等效于 macOS 部分应用的「点标题栏回主界面」习惯。
 */
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
  </div>
</template>

<style lang="less" scoped>
/* 主窗口容器：全屏毛玻璃卡片，内含标题栏 + 内容区 */
.void-window {
  width: 100vw;
  height: 100vh;
  border-radius: var(--void-radius);
  background: var(--void-bg);
  backdrop-filter: blur(40px);
  -webkit-backdrop-filter: blur(40px);
  border: 1px solid var(--void-border);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
