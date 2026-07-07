<script setup lang="ts">
/**
 * index.vue
 * 图片编辑器 — 上传图片并进入编辑会话
 */
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Check, ChevronLeft, ImagePlus, Loader2 } from '@lucide/vue'
import { useImageEditor } from '@/composables/useImageEditor'
import EditorSession from './editor-session.vue'

const router = useRouter()
const fileInputRef = ref<HTMLInputElement | null>(null)
const dragOver = ref(false)

const {
  loadedImage,
  sessionActive,
  loading,
  exportFormatId,
  exportFormats,
  errorMsg,
  successMsg,
  successAction,
  handleFileSelect,
  handleStartEdit,
  handleExitEdit,
  handleExportCanvas,
  handleToastMouseEnter,
  handleToastMouseLeave,
  runSuccessAction,
} = useImageEditor()

const openFilePicker = (): void => {
  fileInputRef.value?.click()
}

const onFileInputChange = (event: Event): void => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  void handleFileSelect(file)
  input.value = ''
}

const onDrop = (event: DragEvent): void => {
  event.preventDefault()
  dragOver.value = false
  const file = event.dataTransfer?.files?.[0]
  void handleFileSelect(file)
}

const onDragOver = (event: DragEvent): void => {
  event.preventDefault()
  dragOver.value = true
}

const onDragLeave = (): void => {
  dragOver.value = false
}
</script>

<template>
  <div class="image-tool">
    <button class="image-tool__back" aria-label="返回首页" @click="router.push('/')">
      <ChevronLeft :size="16" :stroke-width="2" />
      返回
    </button>

    <h2 class="image-tool__title">图片编辑器</h2>

    <div
      class="image-tool__dropzone"
      :class="{ 'image-tool__dropzone--active': dragOver, 'image-tool__dropzone--loaded': !!loadedImage }"
      @click="openFilePicker"
      @drop="onDrop"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
    >
      <input
        ref="fileInputRef"
        type="file"
        accept="image/*"
        class="image-tool__file-input"
        @change="onFileInputChange"
      />
      <Loader2 v-if="loading" :size="28" :stroke-width="2" class="image-tool__spin" />
      <ImagePlus v-else :size="28" :stroke-width="1.75" class="image-tool__drop-icon" />
      <p class="image-tool__drop-title">
        {{ loadedImage ? loadedImage.name : '点击或拖拽上传图片' }}
      </p>
      <p v-if="loadedImage" class="image-tool__drop-meta">
        {{ loadedImage.width }} × {{ loadedImage.height }} px
      </p>
      <p v-else class="image-tool__drop-hint">支持 PNG、JPEG、WebP 等常见格式</p>
    </div>

    <p class="image-tool__note">
      裁剪后保留原图，可链式多次裁剪；导出支持多种图片格式。
    </p>

    <button
      class="image-tool__btn"
      :disabled="!loadedImage || loading || sessionActive"
      @click="handleStartEdit"
    >
      开始编辑
    </button>
  </div>

  <Teleport to="body">
    <Transition name="image-tool-toast">
      <div
        v-if="successMsg || errorMsg"
        class="image-tool__toast"
        :class="{
          'image-tool__toast--success': !!successMsg,
          'image-tool__toast--error': !!errorMsg,
          'image-tool__toast--action': !!successAction,
        }"
        :role="errorMsg ? 'alert' : 'status'"
        :aria-live="errorMsg ? 'assertive' : 'polite'"
        @mouseenter="handleToastMouseEnter"
        @mouseleave="handleToastMouseLeave"
      >
        <Check v-if="successMsg" :size="16" :stroke-width="2.5" />
        <span class="image-tool__toast-text">{{ successMsg || errorMsg }}</span>
        <button
          v-if="successAction"
          type="button"
          class="image-tool__toast-action"
          @click="runSuccessAction"
        >
          {{ successAction.label }}
        </button>
      </div>
    </Transition>
  </Teleport>

  <EditorSession
    v-if="sessionActive && loadedImage"
    :file="loadedImage.file"
    :image-name="loadedImage.name"
    :image-width="loadedImage.width"
    :image-height="loadedImage.height"
    v-model:export-format-id="exportFormatId"
    :export-formats="exportFormats"
    @export="handleExportCanvas"
    @exit="handleExitEdit"
  />
</template>

<style lang="less" scoped>
.image-tool {
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  height: calc(100vh - 36px);
  overflow-y: auto;

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

  &__dropzone {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 180px;
    padding: 24px;
    border-radius: 12px;
    border: 1px dashed var(--void-border);
    background: rgba(255, 255, 255, 0.03);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    flex-shrink: 0;

    &:hover,
    &--active {
      border-color: var(--void-accent-dim);
      background: rgba(255, 255, 255, 0.06);
    }

    &--loaded {
      border-style: solid;
    }
  }

  &__file-input {
    display: none;
  }

  &__drop-icon {
    color: var(--void-text-dim);
  }

  &__drop-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--void-text);
    text-align: center;
    word-break: break-all;
  }

  &__drop-meta,
  &__drop-hint {
    margin: 0;
    font-size: 12px;
    color: var(--void-text-dim);
    text-align: center;
  }

  &__note {
    margin: 0;
    font-size: 12px;
    line-height: 1.5;
    color: var(--void-text-dim);
    flex-shrink: 0;
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
    margin-top: auto;

    &:disabled {
      opacity: 0.6;
      cursor: not-allowed;
    }
  }

  &__spin {
    color: var(--void-accent);
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
    pointer-events: auto;
    transform: translateX(-50%);

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
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

:global(.image-tool-toast-enter-active),
:global(.image-tool-toast-leave-active) {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

:global(.image-tool-toast-enter-from),
:global(.image-tool-toast-leave-to) {
  opacity: 0;
  transform: translateX(-50%) translateY(12px);
}

:global(.image-tool-toast-enter-to),
:global(.image-tool-toast-leave-from) {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}
</style>
