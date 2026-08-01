<script setup lang="ts">
/**
 * VoidButton.vue
 * 应用标准按钮
 *
 * - kind：button（普通）/ icon（正方形图标）
 * - size：small · compact · medium · large · xlarge
 * - variant：配色
 */
import { computed } from 'vue'

export type VoidButtonVariant = 'primary' | 'secondary' | 'accent' | 'danger' | 'ghost'
export type VoidButtonKind = 'button' | 'icon'
export type VoidButtonSize = 'small' | 'compact' | 'medium' | 'large' | 'xlarge'

const props = withDefaults(
  defineProps<{
    variant?: VoidButtonVariant
    kind?: VoidButtonKind
    size?: VoidButtonSize
    block?: boolean
    grow?: boolean
    disabled?: boolean
    loading?: boolean
    active?: boolean
    htmlType?: 'button' | 'submit' | 'reset'
  }>(),
  {
    variant: 'primary',
    kind: 'button',
    size: 'medium',
    block: false,
    grow: false,
    disabled: false,
    loading: false,
    active: false,
    htmlType: 'button',
  },
)

/** icon 默认不用实心主色，仅 grow + primary 时保留（如导出） */
const resolvedVariant = computed((): VoidButtonVariant => {
  if (props.kind === 'icon' && props.variant === 'primary' && !props.grow) {
    return 'secondary'
  }
  return props.variant
})
</script>

<template>
  <button
    :type="htmlType"
    class="void-btn"
    :class="[
      `void-btn--${resolvedVariant}`,
      `void-btn--kind-${kind}`,
      `void-btn--size-${size}`,
      {
        'void-btn--block': block,
        'void-btn--grow': grow,
        'void-btn--loading': loading,
        'void-btn--active': active,
      },
    ]"
    :disabled="disabled || loading"
  >
    <slot />
  </button>
</template>

<style lang="less">
.void-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 20px;
  border-radius: 8px;
  border: none;
  font-size: 14px;
  font-weight: 600;
  line-height: 1;
  cursor: pointer;
  transition: opacity 0.15s, filter 0.15s, background 0.15s, border-color 0.15s, color 0.15s;
  flex-shrink: 0;
  box-sizing: border-box;

  &--block {
    width: 100%;
  }

  &--grow {
    flex: 1;
    min-width: 0;
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  &:hover:not(:disabled) {
    filter: brightness(1.08);
  }

  &--primary:not(.void-btn--kind-icon) {
    background: var(--void-accent);
    color: #000;
  }

  &--secondary:not(.void-btn--kind-icon) {
    background: rgba(255, 255, 255, 0.08);
    color: #e5e7eb;
    border: 1px solid rgba(255, 255, 255, 0.12);
  }

  &--accent:not(.void-btn--kind-icon) {
    background: rgba(192, 132, 252, 0.18);
    color: #e9d5ff;
    border: 1px solid rgba(192, 132, 252, 0.35);
  }

  &--danger:not(.void-btn--kind-icon) {
    background: rgba(255, 59, 48, 0.2);
    color: #ff3b30;
    border: 1px solid rgba(255, 59, 48, 0.3);
  }

  &--ghost {
    min-width: 88px;
    background: var(--void-border);
    color: var(--void-text);

    &:hover:not(:disabled) {
      filter: none;
      background: rgba(255, 255, 255, 0.12);
    }
  }

  &--size-small {
    height: 24px;
    min-height: 24px;
    padding: 0 10px;
    font-size: 12px;
    border-radius: 6px;
  }

  &--size-compact {
    height: 28px;
    min-height: 28px;
    padding: 0 12px;
    font-size: 12px;
    border-radius: 6px;
  }

  &--size-medium {
    height: 32px;
    min-height: 32px;
    padding: 0 14px;
    font-size: 13px;
    border-radius: 6px;
  }

  &--size-large {
    height: 36px;
    min-height: 36px;
    padding: 0 16px;
    font-size: 13px;
  }

  &--size-xlarge {
    height: 40px;
    min-height: 40px;
    padding: 0 20px;
    font-size: 14px;
  }

  &--kind-icon {
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-weight: 400;
    padding: 0;
    gap: 0;

    &:hover:not(:disabled) {
      filter: none;
      background: rgba(255, 255, 255, 0.08);
    }
  }

  &--size-small&--kind-icon {
    width: 24px;
    min-width: 24px;
  }

  &--size-compact&--kind-icon {
    width: 28px;
    min-width: 28px;
  }

  &--size-medium&--kind-icon {
    width: 32px;
    min-width: 32px;
  }

  &--size-large&--kind-icon {
    width: 36px;
    min-width: 36px;
  }

  &--size-xlarge&--kind-icon {
    width: 40px;
    min-width: 40px;
  }

  &--active {
    border-color: rgba(192, 132, 252, 0.5);
    background: rgba(192, 132, 252, 0.15);
    color: #e9d5ff;

    &:hover:not(:disabled) {
      filter: none;
      background: rgba(192, 132, 252, 0.22);
    }
  }
}
</style>
