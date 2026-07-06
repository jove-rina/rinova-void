<script setup lang="ts">
/**
 * AboutDialog.vue
 * 关于 Void 对话框
 */
import { version as appVersion } from '../../package.json'

defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <div v-if="visible" class="about-overlay" @click.self="emit('close')">
    <div class="about-dialog">
      <h2 class="about-dialog__title">Void</h2>
      <p class="about-dialog__version">v{{ appVersion }}</p>
      <p class="about-dialog__desc">空 — 桌面工具箱</p>
      <button type="button" class="about-dialog__btn" @click="emit('close')">确定</button>
    </div>
  </div>
</template>

<style lang="less" scoped>
.about-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
}

.about-dialog {
  width: 280px;
  padding: 28px 24px 20px;
  border-radius: 14px;
  background: var(--void-bg);
  border: 1px solid var(--void-border);
  text-align: center;

  &__title {
    font-size: 22px;
    font-weight: 700;
    color: var(--void-accent);
    letter-spacing: 2px;
    text-transform: uppercase;
  }

  &__version {
    margin-top: 6px;
    font-size: 11px;
    color: var(--void-text-dim);
    opacity: 0.5;
  }

  &__desc {
    margin-top: 16px;
    font-size: 13px;
    color: var(--void-text-dim);
    line-height: 1.5;
  }

  &__btn {
    margin-top: 22px;
    min-width: 88px;
    padding: 8px 16px;
    border: none;
    border-radius: 8px;
    background: var(--void-border);
    color: var(--void-text);
    font-size: 13px;
    cursor: pointer;
    transition: background 0.15s;

    &:hover {
      background: rgba(255, 255, 255, 0.12);
    }
  }
}
</style>
