<script setup lang="ts">
/**
 * Home.vue
 * Void 口袋首页 — 展示注册表中的工具卡片
 *
 * 功能：
 * - 从 registry 读取工具列表并渲染可点击卡片
 * - 每 5 秒轮询各工具运行状态，显示「运行中」徽章
 * - 点击卡片 router.push 到对应工具路由
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { ChevronRight, CircleDot } from '@lucide/vue'
import { isClashServiceRunning } from '@/composables/useClashService'
import { tools } from '@/tools/registry'

const router = useRouter()

/** 当前处于运行状态的工具 id 集合，用于徽章展示 */
const runningTools = ref<Set<string>>(new Set())

/** 轮询定时器句柄，组件卸载时清除 */
let pollTimer: ReturnType<typeof setInterval> | undefined

/**
 * 查询各工具后端状态，更新 runningTools。
 * 目前仅 Clash 服务有运行态探测；后续新工具可在此扩展。
 */
const refreshRunningState = async (): Promise<void> => {
  const next = new Set<string>()
  if (await isClashServiceRunning()) {
    next.add('clash-service')
  }
  runningTools.value = next
}

onMounted(() => {
  refreshRunningState()
  pollTimer = setInterval(refreshRunningState, 5000)
})

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer)
})

/**
 * 导航到指定工具页。
 *
 * @param route - 工具在 registry 中定义的 path
 */
const openTool = (route: string): void => {
  router.push(route)
}
</script>

<template>
  <div class="home">
    <!-- 品牌区：Void 标题与副标题 -->
    <div class="home__pocket">
      <CircleDot :size="36" :stroke-width="1.5" class="home__icon" />
      <h1 class="home__title">Void</h1>
      <p class="home__subtitle">虚空口袋</p>
    </div>

    <!-- 工具卡片列表：数据来自 registry -->
    <div class="home__tools">
      <div
        v-for="tool in tools"
        :key="tool.id"
        class="home__tool-card"
        @click="openTool(tool.route)"
      >
        <component :is="tool.icon" :size="24" :stroke-width="1.75" class="home__tool-icon" />
        <div class="home__tool-body">
          <div class="home__tool-name-row">
            <span class="home__tool-name">{{ tool.name }}</span>
            <span
              v-if="runningTools.has(tool.id)"
              class="home__tool-badge"
            >
              运行中
            </span>
          </div>
          <div class="home__tool-desc">{{ tool.description }}</div>
        </div>
        <ChevronRight :size="20" :stroke-width="2" class="home__tool-arrow" />
      </div>
    </div>

    <div class="home__footer">
      <span class="home__hint">更多工具即将到来...</span>
    </div>
  </div>
</template>

<style lang="less" scoped>
.home {
  display: flex;
  flex-direction: column;
  padding: 0 24px;
  height: calc(100vh - 36px);

  &__pocket {
    text-align: center;
    margin-top: 28px;
    margin-bottom: 20px;
  }

  &__icon {
    display: flex;
    justify-content: center;
    margin-bottom: 6px;
    color: var(--void-accent);
  }

  &__title {
    font-size: 22px;
    font-weight: 700;
    color: var(--void-accent);
    letter-spacing: 2px;
    text-transform: uppercase;
  }

  &__subtitle {
    font-size: 13px;
    color: var(--void-text-dim);
    margin-top: 2px;
  }

  &__tools {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    flex: 1;
    overflow-y: auto;
  }

  &__tool-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border-radius: 12px;
    background: var(--void-border);
    cursor: pointer;
    transition: background 0.15s;

    &:hover {
      background: rgba(255, 255, 255, 0.12);

      .home__tool-arrow {
        opacity: 1;
        transform: translateX(2px);
      }
    }

    &:active {
      background: rgba(255, 255, 255, 0.16);
    }
  }

  &__tool-icon {
    flex-shrink: 0;
    color: var(--void-text);
  }

  &__tool-body {
    flex: 1;
    min-width: 0;
  }

  &__tool-name-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  &__tool-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--void-text);
  }

  &__tool-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(34, 197, 94, 0.15);
    color: #86efac;
    border: 1px solid rgba(34, 197, 94, 0.25);
    flex-shrink: 0;
  }

  &__tool-desc {
    font-size: 12px;
    color: var(--void-text-dim);
    margin-top: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  &__tool-arrow {
    color: var(--void-text-dim);
    opacity: 0.3;
    transition: opacity 0.15s, transform 0.15s;
    flex-shrink: 0;
  }

  &__footer {
    padding: 16px 0;
    text-align: center;
  }

  &__hint {
    font-size: 11px;
    color: var(--void-text-dim);
    opacity: 0.4;
  }
}
</style>
