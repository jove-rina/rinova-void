<script setup lang="ts">
/**
 * SettingsDialog.vue
 * 应用设置面板
 */
import { ref } from 'vue'
import { Info, RotateCcw } from '@lucide/vue'
import VoidButton from '@/components/VoidButton.vue'
import { resetMainWindow } from '@/api/window'

defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
  'show-about': []
}>()

const resetting = ref(false)

const handleResetWindow = async (): Promise<void> => {
  resetting.value = true
  try {
    await resetMainWindow()
    emit('close')
  } catch {
    // 非 Tauri 环境或 IPC 失败时静默
  } finally {
    resetting.value = false
  }
}

const handleShowAbout = (): void => {
  emit('close')
  emit('show-about')
}
</script>

<template>
  <div v-if="visible" class="settings-overlay" @click.self="emit('close')">
    <div class="settings-dialog" role="dialog" aria-labelledby="settings-title">
      <h2 id="settings-title" class="settings-dialog__title">设置</h2>

      <div class="settings-dialog__list">
        <button
          type="button"
          class="settings-dialog__item"
          :disabled="resetting"
          @click="handleResetWindow"
        >
          <RotateCcw :size="16" :stroke-width="2" class="settings-dialog__item-icon" />
          <span class="settings-dialog__item-label">重置窗口</span>
          <span class="settings-dialog__item-hint">恢复默认尺寸与位置</span>
        </button>

        <button type="button" class="settings-dialog__item" @click="handleShowAbout">
          <Info :size="16" :stroke-width="2" class="settings-dialog__item-icon" />
          <span class="settings-dialog__item-label">关于 Void</span>
        </button>
      </div>

      <VoidButton variant="ghost" size="medium" class="settings-dialog__close" @click="emit('close')">
        关闭
      </VoidButton>
    </div>
  </div>
</template>

<style lang="less">
.settings-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
}

.settings-dialog {
  width: min(320px, calc(100vw - 48px));
  padding: 20px 18px 16px;
  border-radius: 14px;
  background: var(--void-bg);
  border: 1px solid var(--void-border);

  &__title {
    margin: 0 0 14px;
    font-size: 16px;
    font-weight: 600;
    color: var(--void-text);
  }

  &__list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  &__item {
    display: grid;
    grid-template-columns: 20px 1fr;
    grid-template-rows: auto auto;
    column-gap: 10px;
    row-gap: 2px;
    align-items: center;
    width: 100%;
    padding: 10px 12px;
    border: 1px solid transparent;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    color: var(--void-text);
    text-align: left;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;

    &:hover:not(:disabled) {
      background: rgba(255, 255, 255, 0.07);
      border-color: var(--void-border);
    }

    &:disabled {
      opacity: 0.6;
      cursor: wait;
    }
  }

  &__item-icon {
    grid-row: 1 / span 2;
    color: var(--void-accent);
  }

  &__item-label {
    font-size: 14px;
    font-weight: 500;
  }

  &__item-hint {
    grid-column: 2;
    font-size: 11px;
    color: var(--void-text-dim);
  }

  &__close {
    width: 100%;
    margin-top: 14px;
  }
}
</style>
