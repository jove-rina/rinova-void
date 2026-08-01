<script setup lang="ts">
/**
 * ToolEntryLayout.vue
 * 工具入口页标准布局：顶栏（返回 | 图标 + 名称）/ 内容区 / 底栏操作
 */
import { useRouter } from 'vue-router'
import { ChevronLeft } from '@lucide/vue'
import type { LucideIcon } from '@lucide/vue'

const props = withDefaults(
  defineProps<{
    /** 工具名称，显示在顶栏 */
    title: string
    /** 工具 Lucide 图标组件 */
    icon: LucideIcon
    /** 返回按钮跳转路径 */
    backRoute?: string
  }>(),
  {
    backRoute: '/',
  },
)

const router = useRouter()

const handleBack = (): void => {
  if (router.currentRoute.value.path !== props.backRoute) {
    router.push(props.backRoute)
  }
}
</script>

<template>
  <div class="tool-entry">
    <header class="tool-entry__top">
      <button class="tool-entry__back" aria-label="返回首页" @click="handleBack">
        <ChevronLeft :size="16" :stroke-width="2" />
        返回
      </button>
      <span class="tool-entry__sep" aria-hidden="true">|</span>
      <div class="tool-entry__brand">
        <component :is="icon" :size="18" :stroke-width="2" class="tool-entry__brand-icon" />
        <h2 class="tool-entry__title">{{ title }}</h2>
      </div>
    </header>

    <section class="tool-entry__body">
      <slot />
    </section>

    <footer v-if="$slots.foot" class="tool-entry__foot">
      <slot name="foot" />
    </footer>
  </div>
</template>

<style lang="less">
.tool-entry {
  box-sizing: border-box;
  flex: 1;
  min-height: 0;
  padding: 0 clamp(16px, 3vw, 24px) clamp(12px, 2vh, 20px);
  display: flex;
  flex-direction: column;
  gap: clamp(8px, 1.5vh, 12px);
  overflow: hidden;

  &__top {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    min-height: 32px;
    gap: clamp(8px, 1.5vw, 12px);
    min-width: 0;
  }

  &__back {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    height: 32px;
    padding: 0;
    border: none;
    background: none;
    color: var(--void-text-dim);
    font-size: 13px;
    line-height: 1;
    cursor: pointer;
    transition: color 0.15s;
    flex-shrink: 0;

    &:hover {
      color: var(--void-accent);
    }
  }

  &__sep {
    display: inline-flex;
    align-items: center;
    align-self: stretch;
    flex-shrink: 0;
    color: rgba(255, 255, 255, 0.18);
    font-size: 14px;
    line-height: 1;
    user-select: none;
  }

  &__brand {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
    height: 32px;
  }

  &__brand-icon {
    flex-shrink: 0;
    display: block;
    color: var(--void-accent);
  }

  &__title {
    margin: 0;
    font-size: clamp(15px, 2.2vw, 16px);
    font-weight: 600;
    line-height: 1;
    color: var(--void-accent);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: clamp(8px, 1.2vh, 10px);
    overflow: hidden;
  }

  &__foot {
    flex: 0 0 auto;
    min-width: 0;
  }
}

.tool-entry__note {
  flex: 0 0 auto;
  margin: 0;
  font-size: clamp(11px, 1.8vw, 12px);
  line-height: 1.45;
  color: var(--void-text-dim);
}
</style>
