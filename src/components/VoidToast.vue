<script setup lang="ts">
/**
 * VoidToast.vue
 * 应用级 Toast 提示 — 成功 / 失败 / 警告 / 通知
 */
import { computed, type Component } from 'vue'
import { AlertTriangle, Check, CircleX, Info } from '@lucide/vue'
import type { ToastType, UseToastReturn } from '@/composables/useToast'

const props = defineProps<{
  controller: UseToastReturn
}>()

const toast = computed(() => props.controller.state.value)

const iconMap: Record<ToastType, Component> = {
  success: Check,
  error: CircleX,
  warning: AlertTriangle,
  info: Info,
}

const isAlert = computed(() => toast.value?.type === 'error')
</script>

<template>
  <Teleport to="body">
    <Transition name="void-toast">
      <div
        v-if="toast"
        class="void-toast"
        :class="[
          `void-toast--${toast.type}`,
          { 'void-toast--action': !!toast.action },
        ]"
        :role="isAlert ? 'alert' : 'status'"
        :aria-live="isAlert ? 'assertive' : 'polite'"
        @mouseenter="controller.onMouseEnter"
        @mouseleave="controller.onMouseLeave"
      >
        <component
          :is="iconMap[toast.type]"
          :size="16"
          :stroke-width="2.5"
          class="void-toast__icon"
        />
        <span class="void-toast__text">{{ toast.message }}</span>
        <button
          v-if="toast.action"
          type="button"
          class="void-toast__action"
          @click="controller.runAction"
        >
          {{ toast.action.label }}
        </button>
      </div>
    </Transition>
  </Teleport>
</template>

<style lang="less">
.void-toast {
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
  pointer-events: auto;
  transform: translateX(-50%);
  background: rgba(22, 23, 29, 0.96);

  &__icon {
    flex-shrink: 0;
  }

  &__text {
    flex: 1;
    min-width: 0;
  }

  &__action {
    flex-shrink: 0;
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    background: transparent;
  }

  &--success {
    border: 1px solid rgba(34, 197, 94, 0.35);
    color: #86efac;

    .void-toast__action {
      border: 1px solid rgba(134, 239, 172, 0.35);
      background: rgba(34, 197, 94, 0.12);
      color: #86efac;

      &:hover {
        background: rgba(34, 197, 94, 0.22);
      }
    }
  }

  &--error {
    border: 1px solid rgba(239, 68, 68, 0.35);
    color: #fca5a5;
  }

  &--warning {
    border: 1px solid rgba(245, 158, 11, 0.35);
    color: #fcd34d;
  }

  &--info {
    border: 1px solid rgba(96, 165, 250, 0.35);
    color: #93c5fd;

    .void-toast__action {
      border: 1px solid rgba(147, 197, 253, 0.35);
      background: rgba(59, 130, 246, 0.12);
      color: #93c5fd;

      &:hover {
        background: rgba(59, 130, 246, 0.22);
      }
    }
  }
}

.void-toast-enter-active,
.void-toast-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.void-toast-enter-from,
.void-toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(12px);
}

.void-toast-enter-to,
.void-toast-leave-from {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}
</style>
