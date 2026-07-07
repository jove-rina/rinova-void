<script setup lang="ts">
/**
 * index.vue
 * 取色器工具页 — 取色记录 + 启动截屏取色
 */
import { computed, onMounted, ref, watch } from 'vue'
import { Loader2, Pipette } from '@lucide/vue'
import ToolEntryLayout from '@/components/ToolEntryLayout.vue'
import VoidButton from '@/components/VoidButton.vue'
import VoidToast from '@/components/VoidToast.vue'
import { useColorPicker } from '@/composables/useColorPicker'
import { consumeColorPickerAutoStart, pendingColorPickerAutoStart } from '@/utils/color-picker-launch'
import ColorRecordsSection from './color-records-section.vue'
import PickerSession from './picker-session.vue'

const {
  startRadius,
  hideAppOnCapture,
  selectedMonitorIndex,
  captureAllScreens,
  monitors,
  monitorsLoading,
  capturing,
  refreshingCapture,
  session,
  records,
  toast,
  magnifyOptions,
  handleStartPick,
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

const recordsExpanded = ref(false)

const recordsFill = computed(() => recordsExpanded.value && records.value.length > 0)

const tryAutoStartPick = (): void => {
  if (!consumeColorPickerAutoStart()) return
  if (capturing.value || session.value) return
  void handleStartPick()
}

onMounted(() => {
  tryAutoStartPick()
})

watch(pendingColorPickerAutoStart, (pending) => {
  if (pending) tryAutoStartPick()
})
</script>

<template>
  <ToolEntryLayout title="取色器" :icon="Pipette" class="color-tool">
    <div class="color-tool__body" :class="{ 'color-tool__body--records-expanded': recordsFill }">
      <ColorRecordsSection
        v-model:expanded="recordsExpanded"
        :records="records"
        :fill="recordsFill"
        empty-text="还没有取色记录，点击下方「开始取色」从屏幕采集颜色。"
        @rename="handleRenameRecord"
        @delete-record="handleRemoveRecord"
        @copy-hex="handleCopyRecordHex"
        @copy-rgb="handleCopyRecordRgb"
        @copy-hsl="handleCopyRecordHsl"
        @export="handleExportRecords"
      />

      <template v-if="!recordsFill">
        <label class="color-tool__select-label">
          目标显示器
          <select
            v-model.number="selectedMonitorIndex"
            class="color-tool__select void-select"
            :disabled="capturing || monitorsLoading || !monitors.length || captureAllScreens"
          >
            <option v-for="mon in monitors" :key="mon.index" :value="mon.index">
              {{ mon.label }}
            </option>
          </select>
        </label>

        <label class="color-tool__option">
          <input v-model="captureAllScreens" type="checkbox" :disabled="capturing" />
          截取全部屏幕
        </label>

        <label class="color-tool__select-label">
          放大倍数
          <select v-model.number="startRadius" class="color-tool__select void-select" :disabled="capturing">
            <option v-for="opt in magnifyOptions" :key="opt.radius" :value="opt.radius">
              {{ opt.label }}
            </option>
          </select>
        </label>

        <label class="color-tool__option">
          <input v-model="hideAppOnCapture" type="checkbox" :disabled="capturing" />
          截屏时隐藏应用窗口
        </label>
      </template>
    </div>

    <template #foot>
      <VoidButton block size="xlarge" :disabled="capturing || !!session" :loading="capturing" @click="handleStartPick">
        <Loader2 v-if="capturing" :size="16" :stroke-width="2" class="color-tool__spin" />
        <Pipette v-else :size="16" :stroke-width="2" />
        {{ capturing ? '正在截屏…' : '开始取色' }}
      </VoidButton>
    </template>
  </ToolEntryLayout>

  <VoidToast :controller="toast" />

  <Teleport to="body">
    <div v-if="capturing" class="color-tool__capture-overlay" role="status" aria-live="polite">
      <Loader2 :size="36" :stroke-width="2" class="color-tool__capture-spin" />
      <p class="color-tool__capture-text">正在截取屏幕…</p>
      <p class="color-tool__capture-sub">请稍候，窗口可能短暂隐藏</p>
    </div>
  </Teleport>

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
.color-tool {
  &__body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow: hidden;

    &--records-expanded {
      gap: 0;
    }
  }

  &__select-label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    color: var(--void-text-dim);
    flex-shrink: 0;
  }

  &__select {
    padding: 8px 10px;
    border-radius: 8px;
    border: 1px solid var(--void-border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--void-text);
    font-size: 13px;
    outline: none;

    &:focus {
      border-color: var(--void-accent-dim);
    }
  }

  &__option {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--void-text-dim);
    cursor: pointer;
    user-select: none;
    flex-shrink: 0;

    input {
      accent-color: var(--void-accent);
    }
  }

  &__spin {
    animation: spin 1s linear infinite;
  }

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
