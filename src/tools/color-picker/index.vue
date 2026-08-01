<script setup lang="ts">
/**
 * index.vue
 * 取色器工具页 — 取色记录 + 启动独立取色窗口
 */
import { computed, onMounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Loader2, Pipette } from '@lucide/vue'
import ToolEntryLayout from '@/components/ToolEntryLayout.vue'
import VoidButton from '@/components/VoidButton.vue'
import VoidToast from '@/components/VoidToast.vue'
import { useColorPicker } from '@/composables/useColorPicker'
import { loadColorRecords } from '@/utils/color-records'
import ColorRecordsSection from './color-records-section.vue'

const {
  startRadius,
  hideAppOnCapture,
  selectedMonitorIndex,
  captureAllScreens,
  monitors,
  monitorsLoading,
  records,
  toast,
  magnifyOptions,
  handleStartPick,
  handleRemoveRecord,
  handleRenameRecord,
  handleCopyRecordHex,
  handleCopyRecordRgb,
  handleCopyRecordHsl,
  handleExportRecords,
} = useColorPicker()

const launching = ref(false)
const recordsExpanded = ref(false)

const recordsFill = computed(() => recordsExpanded.value && records.value.length > 0)

const reloadRecords = (): void => {
  records.value = loadColorRecords()
}

const handleLaunchPick = async (): Promise<void> => {
  launching.value = true
  try {
    await handleStartPick()
  } finally {
    launching.value = false
  }
}

onMounted(async () => {
  try {
    await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) reloadRecords()
    })
  } catch {
    // 非 Tauri 环境
  }
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
            :disabled="launching || monitorsLoading || !monitors.length || captureAllScreens"
          >
            <option v-for="mon in monitors" :key="mon.index" :value="mon.index">
              {{ mon.label }}
            </option>
          </select>
        </label>

        <label class="color-tool__option">
          <input v-model="captureAllScreens" type="checkbox" :disabled="launching" />
          截取全部屏幕
        </label>

        <label class="color-tool__select-label">
          放大倍数
          <select v-model.number="startRadius" class="color-tool__select void-select" :disabled="launching">
            <option v-for="opt in magnifyOptions" :key="opt.radius" :value="opt.radius">
              {{ opt.label }}
            </option>
          </select>
        </label>

        <label class="color-tool__option">
          <input v-model="hideAppOnCapture" type="checkbox" :disabled="launching" />
          截屏时隐藏应用窗口
        </label>
      </template>
    </div>

    <template #foot>
      <VoidButton block size="xlarge" :disabled="launching" :loading="launching" @click="handleLaunchPick">
        <Loader2 v-if="launching" :size="16" :stroke-width="2" class="color-tool__spin" />
        <Pipette v-else :size="16" :stroke-width="2" />
        {{ launching ? '正在打开…' : '开始取色' }}
      </VoidButton>
    </template>
  </ToolEntryLayout>

  <VoidToast :controller="toast" />
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
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
