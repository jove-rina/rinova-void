<script setup lang="ts">
/**
 * index.vue
 * 取色器工具页 — 取色记录 + 启动截屏取色
 */
import { onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  Check,
  ChevronLeft,
  Loader2,
  Pipette,
} from '@lucide/vue'
import { useColorPicker } from '@/composables/useColorPicker'
import { consumeColorPickerAutoStart, pendingColorPickerAutoStart } from '@/utils/color-picker-launch'
import ColorRecordsSection from './color-records-section.vue'
import PickerSession from './picker-session.vue'

const router = useRouter()

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
  errorMsg,
  successMsg,
  successAction,
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
  runSuccessAction,
  handleSessionRadiusChange,
  handleExitPick,
} = useColorPicker()

const recordsExpanded = ref(false)

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
  <div
    class="color-tool"
    :class="{ 'color-tool--records-expanded': recordsExpanded && records.length > 0 }"
  >
    <button class="color-tool__back" aria-label="返回首页" @click="router.push('/')">
      <ChevronLeft :size="16" :stroke-width="2" />
      返回
    </button>

    <h2 class="color-tool__title">取色器</h2>

    <ColorRecordsSection
      v-model:expanded="recordsExpanded"
      :records="records"
      :fill="recordsExpanded && records.length > 0"
      empty-text="还没有取色记录，点击下方「开始取色」从屏幕采集颜色。"
      @rename="handleRenameRecord"
      @delete-record="handleRemoveRecord"
      @copy-hex="handleCopyRecordHex"
      @copy-rgb="handleCopyRecordRgb"
      @copy-hsl="handleCopyRecordHsl"
      @export="handleExportRecords"
    />

    <template v-if="!recordsExpanded || records.length === 0">
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

    <button
      class="color-tool__btn"
      :class="{ 'color-tool__btn--pinned': recordsExpanded && records.length > 0 }"
      :disabled="capturing || !!session"
      @click="handleStartPick"
    >
      <Loader2 v-if="capturing" :size="16" :stroke-width="2" class="color-tool__spin" />
      <Pipette v-else :size="16" :stroke-width="2" />
      {{ capturing ? '正在截屏…' : '开始取色' }}
    </button>
  </div>

  <Teleport to="body">
    <Transition name="color-tool-toast">
      <div
        v-if="successMsg"
        class="color-tool__toast color-tool__toast--success"
        :class="{ 'color-tool__toast--action': successAction }"
        role="status"
        aria-live="polite"
      >
        <Check :size="16" :stroke-width="2.5" />
        <span class="color-tool__toast-text">{{ successMsg }}</span>
        <button
          v-if="successAction"
          type="button"
          class="color-tool__toast-action"
          @click="runSuccessAction"
        >
          {{ successAction.label }}
        </button>
      </div>
    </Transition>
    <Transition name="color-tool-toast">
      <div
        v-if="errorMsg"
        class="color-tool__toast color-tool__toast--error"
        role="alert"
        aria-live="assertive"
      >
        <span>{{ errorMsg }}</span>
      </div>
    </Transition>
  </Teleport>

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
    @exit="handleExitPick"
  />
</template>

<style lang="less" scoped>
.color-tool {
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  height: calc(100vh - 36px);
  overflow-y: auto;

  &--records-expanded {
    overflow: hidden;
  }

  &__back {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    align-self: flex-start;
    padding: 4px 0;
    border: none;
    background: none;
    color: var(--void-text-dim);
    font-size: 13px;
    cursor: pointer;
    transition: color 0.15s;
    flex-shrink: 0;

    &:hover {
      color: var(--void-accent);
    }
  }

  &__title {
    font-size: 16px;
    font-weight: 600;
    color: var(--void-accent);
    margin-bottom: 0;
    flex-shrink: 0;
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

  &__btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px 20px;
    border-radius: 8px;
    border: none;
    background: var(--void-accent);
    color: #000;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s;
    flex-shrink: 0;

    &--pinned {
      margin-top: auto;
    }

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
  }

  &__spin {
    animation: spin 1s linear infinite;
  }

  &__toast {
    position: fixed;
    left: 50%;
    bottom: 28px;
    z-index: 10002;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    max-width: min(420px, calc(100vw - 32px));
    padding: 10px 14px;
    border-radius: 10px;
    font-size: 13px;
    font-weight: 500;
    line-height: 1.4;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    pointer-events: none;
    transform: translateX(-50%);

    &--action {
      pointer-events: auto;
    }

    &--success {
      background: rgba(22, 23, 29, 0.96);
      border: 1px solid rgba(34, 197, 94, 0.35);
      color: #86efac;
    }

    &--error {
      background: rgba(22, 23, 29, 0.96);
      border: 1px solid rgba(239, 68, 68, 0.35);
      color: #fca5a5;
    }
  }

  &__toast-text {
    flex: 1;
    min-width: 0;
  }

  &__toast-action {
    flex-shrink: 0;
    padding: 4px 10px;
    border: 1px solid rgba(134, 239, 172, 0.35);
    border-radius: 6px;
    background: rgba(34, 197, 94, 0.12);
    color: #86efac;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;

    &:hover {
      background: rgba(34, 197, 94, 0.22);
    }
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

:global(.color-tool-toast-enter-active),
:global(.color-tool-toast-leave-active) {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

:global(.color-tool-toast-enter-from),
:global(.color-tool-toast-leave-to) {
  opacity: 0;
  transform: translateX(-50%) translateY(12px);
}

:global(.color-tool-toast-enter-to),
:global(.color-tool-toast-leave-from) {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}
</style>
