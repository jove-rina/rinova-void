<script setup lang="ts">
/**
 * index.vue
 * 图片编辑器 — 多图上传、项目、打开独立编辑窗口
 */
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { FolderOpen, Image, ImagePlus, Loader2, Trash2, X } from '@lucide/vue'
import ToolEntryLayout from '@/components/ToolEntryLayout.vue'
import VoidButton from '@/components/VoidButton.vue'
import VoidToast from '@/components/VoidToast.vue'
import { useImageEditor } from '@/composables/useImageEditor'
import { formatProjectTime } from '@/utils/image-editor-project'

const fileInputRef = ref<HTMLInputElement | null>(null)
const launching = ref(false)

const isTauri = (): boolean =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

const {
  entryImages,
  activeEntryId,
  activeEntry,
  previewUrl,
  savedProjects,
  loading,
  dragOver,
  toast,
  handleFilesSelect,
  handlePathSelect,
  selectEntryImage,
  removeEntryImage,
  handleStartEdit,
  handleOpenProject,
  handleDeleteProject,
  refreshProjects,
} = useImageEditor()

const handleLaunchEdit = async (): Promise<void> => {
  launching.value = true
  try {
    await handleStartEdit()
  } finally {
    launching.value = false
  }
}

const handleLaunchProject = async (projectId: string): Promise<void> => {
  launching.value = true
  try {
    await handleOpenProject(projectId)
  } finally {
    launching.value = false
  }
}

let unlistenDragDrop: UnlistenFn | undefined

const openFilePicker = (): void => {
  fileInputRef.value?.click()
}

const onFileInputChange = (event: Event): void => {
  const input = event.target as HTMLInputElement
  void handleFilesSelect(input.files)
  input.value = ''
}

const onDrop = (event: DragEvent): void => {
  event.preventDefault()
  dragOver.value = false
  if (isTauri()) return
  void handleFilesSelect(event.dataTransfer?.files)
}

const onDragOver = (event: DragEvent): void => {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy'
  }
  dragOver.value = true
}

const onDragLeave = (): void => {
  dragOver.value = false
}

onMounted(async () => {
  await refreshProjects()
  if (!isTauri()) return
  try {
    await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) void refreshProjects()
    })
    unlistenDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'over' || event.payload.type === 'enter') {
        dragOver.value = true
        return
      }
      if (event.payload.type === 'leave') {
        dragOver.value = false
        return
      }
      if (event.payload.type === 'drop') {
        const paths = event.payload.paths.filter((p) =>
          /\.(png|jpe?g|gif|webp|bmp|tiff?|ico|avif|svg)$/i.test(p),
        )
        uploadDragPaths(paths)
        dragOver.value = false
      }
    })
  } catch {
    // 非 Tauri 环境
  }
})

const uploadDragPaths = (paths: string[]): void => {
  if (paths.length === 0) {
    showDragError()
    return
  }
  void (async () => {
    for (const path of paths) {
      await handlePathSelect(path)
    }
  })()
}

const showDragError = (): void => {
  dragOver.value = false
}

onUnmounted(() => {
  void unlistenDragDrop?.()
  unlistenDragDrop = undefined
})
</script>

<template>
  <ToolEntryLayout title="图片编辑器" :icon="Image" class="image-tool">
    <div
      class="image-tool__dropzone"
      :class="{
        'image-tool__dropzone--active': dragOver,
        'image-tool__dropzone--loaded': !!previewUrl,
        'image-tool__dropzone--compact': entryImages.length > 0 && savedProjects.length > 0,
      }"
      @click="openFilePicker"
      @drop="onDrop"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
    >
      <input
        ref="fileInputRef"
        type="file"
        accept="image/*"
        multiple
        class="image-tool__file-input"
        @change="onFileInputChange"
      />

      <img
        v-if="previewUrl && !loading"
        :src="previewUrl"
        :alt="activeEntry?.name ?? '图片预览'"
        class="image-tool__preview"
      />

      <div v-if="loading" class="image-tool__overlay">
        <Loader2 :size="28" :stroke-width="2" class="image-tool__spin" />
      </div>

      <div v-if="!previewUrl && !loading" class="image-tool__placeholder">
        <ImagePlus :size="28" :stroke-width="1.75" class="image-tool__drop-icon" />
        <p class="image-tool__drop-title">点击或拖拽上传图片</p>
        <p class="image-tool__drop-hint">支持多选 · PNG、JPEG、WebP 等常见格式</p>
      </div>

      <div v-if="previewUrl && activeEntry" class="image-tool__meta-bar">
        <span class="image-tool__meta-name">{{ activeEntry.name }}</span>
        <span class="image-tool__meta-size">{{ activeEntry.width }} × {{ activeEntry.height }}</span>
      </div>
    </div>

    <div
      v-if="entryImages.length > 0 || savedProjects.length > 0"
      class="image-tool__panels"
      :class="{
        'image-tool__panels--queue': entryImages.length > 0,
        'image-tool__panels--projects': savedProjects.length > 0,
        'image-tool__panels--both': entryImages.length > 0 && savedProjects.length > 0,
      }"
    >
      <div v-if="entryImages.length > 0" class="image-tool__queue">
        <span class="image-tool__queue-label">已上传 {{ entryImages.length }} 张</span>
        <div class="image-tool__queue-list">
          <div
            v-for="item in entryImages"
            :key="item.id"
            class="image-tool__queue-item"
            :class="{ 'image-tool__queue-item--active': item.id === activeEntryId }"
          >
            <button
              type="button"
              class="image-tool__queue-thumb-btn"
              :title="item.name"
              @click.stop="selectEntryImage(item.id)"
            >
              <img
                v-if="item.thumbUrl"
                :src="item.thumbUrl"
                :alt="item.name"
                class="image-tool__queue-thumb"
              />
            </button>
            <button
              type="button"
              class="image-tool__queue-remove"
              aria-label="删除图片"
              @click.stop="removeEntryImage(item.id)"
            >
              <X :size="12" :stroke-width="2.5" />
            </button>
          </div>
          <button type="button" class="image-tool__queue-add" @click.stop="openFilePicker">
            <ImagePlus :size="16" :stroke-width="2" />
          </button>
        </div>
      </div>

      <section v-if="savedProjects.length > 0" class="image-tool__projects">
        <div class="image-tool__projects-head">
          <FolderOpen :size="14" :stroke-width="2" />
          <span>已保存的项目</span>
        </div>
        <div class="image-tool__projects-list">
          <div
            v-for="project in savedProjects"
            :key="project.id"
            class="image-tool__project-card"
          >
            <button
              type="button"
              class="image-tool__project-open"
              @click="handleLaunchProject(project.id)"
            >
              <img
                v-if="project.previewBase64"
                :src="`data:image/png;base64,${project.previewBase64}`"
                :alt="project.name"
                class="image-tool__project-thumb"
              />
              <span v-else class="image-tool__project-fallback">{{ project.name.slice(0, 1) }}</span>
              <span class="image-tool__project-info">
                <span class="image-tool__project-name">{{ project.name }}</span>
                <span class="image-tool__project-meta">
                  {{ project.imageCount }} 张 · {{ formatProjectTime(project.updatedAt) }}
                </span>
              </span>
            </button>
            <button
              type="button"
              class="image-tool__project-delete"
              aria-label="删除项目"
              @click="handleDeleteProject(project.id)"
            >
              <Trash2 :size="14" :stroke-width="2" />
            </button>
          </div>
        </div>
      </section>
    </div>

    <template #foot>
      <VoidButton block size="xlarge" :disabled="entryImages.length === 0 || loading || launching" :loading="launching" @click="handleLaunchEdit">
        <Loader2 v-if="launching" :size="16" :stroke-width="2" class="image-tool__spin" />
        {{ launching ? '正在打开…' : '开始编辑' }}
      </VoidButton>
    </template>
  </ToolEntryLayout>

  <VoidToast :controller="toast" />
</template>

<style lang="less" scoped>
.image-tool {
  &__dropzone {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: center;
    flex: 1;
    min-height: 0;
    padding: 0;
    border-radius: 12px;
    border: 1px dashed var(--void-border);
    background: rgba(255, 255, 255, 0.03);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    overflow: hidden;

    &:hover,
    &--active {
      border-color: var(--void-accent-dim);
      background: rgba(255, 255, 255, 0.06);
    }

    &--loaded {
      border-style: solid;
    }

    &--compact {
      flex: 1 1 42%;
      min-height: 120px;
    }
  }

  &__file-input {
    display: none;
  }

  &__preview {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
    background: rgba(0, 0, 0, 0.25);
  }

  &__overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.45);
    pointer-events: none;
  }

  &__placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: clamp(4px, 1vh, 8px);
    padding: clamp(12px, 2vh, 24px);
    height: 100%;
    min-height: 0;
  }

  &__drop-icon {
    color: var(--void-text-dim);
    flex-shrink: 0;
  }

  &__drop-title {
    margin: 0;
    font-size: clamp(13px, 2vw, 14px);
    font-weight: 600;
    color: var(--void-text);
    text-align: center;
  }

  &__drop-hint {
    margin: 0;
    font-size: clamp(11px, 1.8vw, 12px);
    color: var(--void-text-dim);
    text-align: center;
  }

  &__meta-bar {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 12px;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.75));
    pointer-events: none;
  }

  &__meta-name {
    font-size: 12px;
    font-weight: 600;
    color: #e5e7eb;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  &__meta-size {
    font-size: 11px;
    color: #9ca3af;
    flex-shrink: 0;
    font-family: ui-monospace, 'Cascadia Code', monospace;
  }

  &__panels {
    flex: 0 1 auto;
    display: flex;
    flex-direction: column;
    gap: clamp(6px, 1vh, 8px);
    min-width: 0;
    min-height: 0;
    max-height: 36%;
    overflow-x: hidden;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.18) transparent;

    &::-webkit-scrollbar {
      width: 4px;
    }

    &::-webkit-scrollbar-thumb {
      border-radius: 4px;
      background: rgba(255, 255, 255, 0.18);
    }

    &--both {
      max-height: 48%;
    }

    &--projects:not(&--queue) {
      max-height: 42%;
    }
  }

  &__queue {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    flex: 0 0 auto;
  }

  &__queue-label {
    font-size: 12px;
    color: var(--void-text-dim);
    flex-shrink: 0;
  }

  &__queue-list,
  &__projects-list {
    display: flex;
    gap: 8px;
    min-width: 0;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    -ms-overflow-style: none;

    &::-webkit-scrollbar {
      display: none;
    }
  }

  &__queue-item {
    position: relative;
    flex: 0 0 auto;
  }

  &__queue-thumb-btn {
    width: clamp(44px, 8vw, 52px);
    height: clamp(44px, 8vw, 52px);
    padding: 0;
    border-radius: 8px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.04);
    overflow: hidden;
    cursor: pointer;
  }

  &__queue-item--active &__queue-thumb-btn {
    border-color: var(--void-accent-dim);
    box-shadow: 0 0 0 1px rgba(192, 132, 252, 0.35);
  }

  &__queue-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  &__queue-remove {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: rgba(15, 15, 20, 0.95);
    color: #fca5a5;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
  }

  &__queue-add {
    flex: 0 0 auto;
    width: clamp(44px, 8vw, 52px);
    height: clamp(44px, 8vw, 52px);
    border-radius: 8px;
    border: 1px dashed rgba(255, 255, 255, 0.18);
    background: rgba(255, 255, 255, 0.03);
    color: var(--void-text-dim);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;

    &:hover {
      color: var(--void-accent);
      border-color: var(--void-accent-dim);
    }
  }

  &__projects {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    flex: 0 0 auto;
  }

  &__projects-head {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--void-text-dim);
    flex-shrink: 0;
  }

  &__project-card {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 min(240px, 78vw);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.03);
    overflow: hidden;
  }

  &__project-open {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border: none;
    background: none;
    cursor: pointer;
    text-align: left;

    &:hover {
      background: rgba(255, 255, 255, 0.04);
    }
  }

  &__project-thumb {
    width: clamp(36px, 6vw, 40px);
    height: clamp(36px, 6vw, 40px);
    border-radius: 6px;
    object-fit: cover;
    flex-shrink: 0;
  }

  &__project-fallback {
    width: clamp(36px, 6vw, 40px);
    height: clamp(36px, 6vw, 40px);
    border-radius: 6px;
    background: rgba(192, 132, 252, 0.15);
    color: #e9d5ff;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    flex-shrink: 0;
  }

  &__project-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  &__project-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--void-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__project-meta {
    font-size: 11px;
    color: var(--void-text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__project-delete {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    margin-right: 4px;
    border: none;
    border-radius: 6px;
    background: none;
    color: #9ca3af;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;

    &:hover {
      color: #fca5a5;
      background: rgba(239, 68, 68, 0.1);
    }
  }

  &__spin {
    color: var(--void-accent);
    animation: spin 1s linear infinite;
  }
}

@media (max-height: 520px) {
  .image-tool__panels {
    max-height: 40%;

    &--both {
      max-height: 52%;
    }
  }

  .image-tool__dropzone {
    min-height: 96px;
  }
}

@media (max-height: 440px) {
  .image-tool__panels {
    max-height: 46%;

    &--both {
      max-height: 58%;
    }
  }

  .image-tool__queue-thumb-btn,
  .image-tool__queue-add {
    width: 40px;
    height: 40px;
  }

  .image-tool__project-card {
    flex-basis: min(200px, 72vw);
  }

  .image-tool__project-thumb,
  .image-tool__project-fallback {
    width: 32px;
    height: 32px;
  }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
