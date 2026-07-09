<script setup lang="ts">
/**
 * session.vue
 * 取色独立窗口 — 全屏取色会话
 */
import { onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Loader2 } from '@lucide/vue'
import VoidToast from '@/components/VoidToast.vue'
import { useColorPicker } from '@/composables/useColorPicker'
import PickerSession from './picker-session.vue'

const {
  capturing,
  refreshingCapture,
  session,
  records,
  monitors,
  toast,
  bootstrapPickerSession,
  handleSessionRefresh,
  handleSessionPick,
  handleRemoveRecord,
  handleRenameRecord,
  handleCopyRecordHex,
  handleCopyRecordRgb,
  handleCopyRecordHsl,
  handleExportRecords,
  handleSessionRadiusChange,
  handleExitPick,
  handleSnapshotLoadError,
} = useColorPicker()

onMounted(async () => {
  const ready = await bootstrapPickerSession()
  if (!ready) {
    await getCurrentWindow().close()
  }
})
</script>

<template>
  <VoidToast :controller="toast" />

  <div
    v-if="capturing"
    class="picker-window__capture-overlay"
    role="status"
    aria-live="polite"
  >
    <Loader2 :size="36" :stroke-width="2" class="picker-window__capture-spin" />
    <p class="picker-window__capture-text">正在截取屏幕…</p>
    <p class="picker-window__capture-sub">请稍候，窗口可能短暂隐藏</p>
  </div>

  <PickerSession
    v-if="session"
    :session="session"
    :records="records"
    :monitors="monitors"
    :refreshing="refreshingCapture"
    @pick="handleSessionPick"
    @rename="handleRenameRecord"
    @delete-record="handleRemoveRecord"
    @copy-hex="handleCopyRecordHex"
    @copy-rgb="handleCopyRecordRgb"
    @copy-hsl="handleCopyRecordHsl"
    @export="handleExportRecords"
    @refresh="handleSessionRefresh"
    @update:radius="handleSessionRadiusChange"
    @load-error="handleSnapshotLoadError"
    @exit="handleExitPick"
  />
</template>

<style lang="less" scoped>
.picker-window {
  &__capture-overlay {
    position: fixed;
    inset: 0;
    z-index: 10001;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    background: rgba(10, 10, 12, 0.92);
    backdrop-filter: blur(4px);
  }

  &__capture-spin {
    color: var(--void-accent);
    animation: spin 0.9s linear infinite;
  }

  &__capture-text {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--void-text);
  }

  &__capture-sub {
    margin: 0;
    font-size: 12px;
    color: var(--void-text-dim);
    opacity: 0.85;
  }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
