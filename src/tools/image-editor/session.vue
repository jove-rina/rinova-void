<script setup lang="ts">
/**
 * session.vue
 * 图片编辑独立窗口 — 多图会话与操作面板
 */
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Loader2 } from '@lucide/vue'
import VoidButton from '@/components/VoidButton.vue'
import VoidToast from '@/components/VoidToast.vue'
import { useToast } from '@/composables/useToast'
import {
  exportImageWithSettings,
  notifyImageEditorProjectsChanged,
  revealExportPath,
  saveImageEditorProject,
  takeImageEditorSession,
  type ExportSettings,
} from '@/api/image-editor'
import {
  createDocumentId,
  revokeThumbUrl,
  type ImageDocumentEntry,
  type ImageDocumentSummary,
  type ImageEditState,
} from '@/utils/image-editor-document'
import {
  createProjectId,
  defaultProjectName,
  fileToBase64,
  sanitizeProjectName,
  type ImageEditorProject,
} from '@/utils/image-editor-project'
import {
  IMAGE_EXPORT_FORMATS,
  IMAGE_COMPRESSION_PRESETS,
  IMAGE_SIZE_SCALE_PRESETS,
  formatInvokeError,
  sanitizeExportStem,
  base64ToBytes,
  uint8ToBase64,
  type ExportMode,
} from '@/utils/image-editor-export'
import { bytesToImageFile, isImageFile, readImageMeta } from '@/utils/image-file-load'
import EditorSession from './editor-session.vue'

type PendingSnapshot = {
  documentId: string
  editState: ImageEditState
  thumbUrl: string
}

const ready = ref(false)
const bootError = ref('')
const activeDocumentId = ref('')
const documents = ref<ImageDocumentEntry[]>([])
const restoredEditState = ref<ImageEditState | null>(null)
const projectId = ref<string | null>(null)
const projectName = ref<string | null>(null)
const saving = ref(false)
const exitDialogVisible = ref(false)
const saveDialogVisible = ref(false)
const saveDialogMode = ref<'save' | 'save-and-exit'>('save')
const saveNameInput = ref('')
const pendingSnapshot = ref<PendingSnapshot | null>(null)
const hasUnsavedChanges = ref(true)
const suppressDirty = ref(true)

const exportMode = ref<ExportMode>('general')
const exportName = ref('')
const exportFormatId = ref(IMAGE_EXPORT_FORMATS[0].id)
const compressionPresetId = ref(IMAGE_COMPRESSION_PRESETS[0].id)
const sizeScalePresetId = ref(IMAGE_SIZE_SCALE_PRESETS[0].id)
const thumbnailSizes = ref<number[]>([])
const icoSizes = ref<number[]>([])
const toast = useToast()

const activeDocument = computed(() =>
  documents.value.find((doc) => doc.id === activeDocumentId.value) ?? null,
)

const documentHistory = computed<ImageDocumentSummary[]>(() =>
  documents.value.map((doc) => ({
    id: doc.id,
    name: doc.name,
    thumbUrl: doc.thumbUrl,
  })),
)

const showSuccess = (message: string, action?: { label: string; run: () => void | Promise<void> }): void => {
  toast.showSuccess(message, action)
}

const showError = (message: string): void => {
  toast.showError(message)
}

const updateDocument = (
  id: string,
  patch: Partial<Pick<ImageDocumentEntry, 'editState' | 'thumbUrl' | 'exportName'>>,
): void => {
  const index = documents.value.findIndex((doc) => doc.id === id)
  if (index < 0) return
  const current = documents.value[index]
  if (patch.thumbUrl && current.thumbUrl) {
    revokeThumbUrl(current.thumbUrl)
  }
  documents.value[index] = { ...current, ...patch }
}

const activateDocument = (id: string): void => {
  const doc = documents.value.find((item) => item.id === id)
  if (!doc) return
  activeDocumentId.value = id
  restoredEditState.value = doc.editState
  exportName.value = doc.exportName
}

const createThumbUrlFromFile = async (file: File): Promise<string> => {
  const bitmap = await createImageBitmap(file)
  const edge = Math.max(bitmap.width, bitmap.height, 1)
  const size = 48
  const scale = size / edge
  const cssW = Math.max(1, Math.round(bitmap.width * scale))
  const cssH = Math.max(1, Math.round(bitmap.height * scale))
  const canvas = document.createElement('canvas')
  canvas.width = cssW
  canvas.height = cssH
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    bitmap.close()
    return ''
  }
  ctx.drawImage(bitmap, 0, 0, cssW, cssH)
  bitmap.close()
  const blob = await new Promise<Blob | null>((resolve) => {
    canvas.toBlob(resolve, 'image/png')
  })
  return blob ? URL.createObjectURL(blob) : ''
}

const addDocumentFromFile = async (
  file: File,
  options?: {
    id?: string
    exportName?: string
    editState?: ImageEditState | null
  },
): Promise<string> => {
  const meta = await readImageMeta(file)
  const thumbUrl = options?.editState
    ? await createThumbUrlFromEditState(options.editState)
    : await createThumbUrlFromFile(file)
  const id = options?.id ?? createDocumentId()
  documents.value.push({
    id,
    name: meta.name,
    file: meta.file,
    width: meta.width,
    height: meta.height,
    exportName: options?.exportName ?? sanitizeExportStem(meta.name),
    editState: options?.editState ?? null,
    thumbUrl,
  })
  return id
}

const createThumbUrlFromEditState = async (editState: ImageEditState): Promise<string> => {
  const bytes = base64ToBytes(editState.workingPng)
  const blob = new Blob([bytes.slice()], { type: 'image/png' })
  const bitmap = await createImageBitmap(blob)
  const edge = Math.max(bitmap.width, bitmap.height, 1)
  const size = 48
  const scale = size / edge
  const cssW = Math.max(1, Math.round(bitmap.width * scale))
  const cssH = Math.max(1, Math.round(bitmap.height * scale))
  const canvas = document.createElement('canvas')
  canvas.width = cssW
  canvas.height = cssH
  const ctx = canvas.getContext('2d')
  if (!ctx) {
    bitmap.close()
    return ''
  }
  ctx.drawImage(bitmap, 0, 0, cssW, cssH)
  bitmap.close()
  const out = await new Promise<Blob | null>((resolve) => {
    canvas.toBlob(resolve, 'image/png')
  })
  return out ? URL.createObjectURL(out) : ''
}

onMounted(async () => {
  try {
    const win = getCurrentWindow()
    await win.show()
    await win.unminimize()
  } catch {
    // 非 Tauri 环境
  }

  try {
    const batch = await takeImageEditorSession()
    if (!batch?.images?.length) {
      bootError.value = '未获取到图片数据，请返回入口页重新点击「开始编辑」。'
      return
    }

    const meta = batch.meta
    exportMode.value = meta.exportMode
    exportFormatId.value = meta.exportFormatId
    compressionPresetId.value = meta.compressionPresetId
    sizeScalePresetId.value = meta.sizeScalePresetId
    thumbnailSizes.value = [...meta.thumbnailSizes]
    icoSizes.value = [...meta.icoSizes]
    projectId.value = meta.projectId
    projectName.value = meta.projectName

    for (const item of batch.images) {
      const file = bytesToImageFile(item.bytes, item.name)
      await addDocumentFromFile(file, {
        id: item.id,
        exportName: item.exportName,
        editState: item.editState,
      })
    }

    const activeId = documents.value.some((doc) => doc.id === meta.activeId)
      ? meta.activeId
      : documents.value[0].id
    activateDocument(activeId)
    hasUnsavedChanges.value = !projectId.value
    ready.value = true
    await nextTick()
    suppressDirty.value = false
  } catch (e) {
    bootError.value = e instanceof Error ? e.message : '图片加载失败'
  }
})

const handleBootErrorClose = async (): Promise<void> => {
  await getCurrentWindow().close()
}

watch(exportName, (name) => {
  if (activeDocumentId.value) {
    updateDocument(activeDocumentId.value, { exportName: name })
  }
  markDirty()
})

const markDirty = (): void => {
  if (suppressDirty.value) return
  hasUnsavedChanges.value = true
}

watch(
  [exportMode, exportFormatId, compressionPresetId, sizeScalePresetId, thumbnailSizes, icoSizes],
  () => markDirty(),
  { deep: true },
)

onUnmounted(() => {
  for (const doc of documents.value) {
    revokeThumbUrl(doc.thumbUrl)
  }
})

const handleExitRequest = async (payload: {
  snapshot: PendingSnapshot | null
}): Promise<void> => {
  if (payload.snapshot) {
    applySnapshot(payload.snapshot)
  }
  if (!hasUnsavedChanges.value) {
    await closeEditorWindow()
    return
  }
  pendingSnapshot.value = payload.snapshot
  exitDialogVisible.value = true
}

const closeExitDialog = (): void => {
  exitDialogVisible.value = false
}

const closeEditorWindow = async (): Promise<void> => {
  await getCurrentWindow().close()
}

const applySnapshot = (snapshot: PendingSnapshot | null | undefined): void => {
  if (!snapshot) return
  updateDocument(snapshot.documentId, {
    editState: snapshot.editState,
    thumbUrl: snapshot.thumbUrl,
  })
}

const openSaveNameDialog = (
  snapshot: PendingSnapshot,
  mode: 'save' | 'save-and-exit',
): void => {
  applySnapshot(snapshot)
  pendingSnapshot.value = snapshot
  saveDialogMode.value = mode
  saveNameInput.value = projectName.value ?? defaultProjectName()
  saveDialogVisible.value = true
}

const closeSaveNameDialog = (): void => {
  saveDialogVisible.value = false
}

const handleExitSave = (): void => {
  const snapshot = pendingSnapshot.value
  closeExitDialog()
  if (!snapshot) {
    showError('无法获取当前编辑状态')
    return
  }
  openSaveNameDialog(snapshot, 'save-and-exit')
}

const handleExitDiscard = async (): Promise<void> => {
  closeExitDialog()
  pendingSnapshot.value = null
  await closeEditorWindow()
}

const handleExitCancel = (): void => {
  closeExitDialog()
  pendingSnapshot.value = null
}

const handleStateSnapshot = (payload: {
  documentId: string
  editState: ImageEditState
  thumbUrl: string
}): void => {
  updateDocument(payload.documentId, {
    editState: payload.editState,
    thumbUrl: payload.thumbUrl,
  })
  markDirty()
}

const handleSwitchDocument = (payload: {
  id: string
  snapshot: { documentId: string; editState: ImageEditState; thumbUrl: string }
}): void => {
  updateDocument(payload.snapshot.documentId, {
    editState: payload.snapshot.editState,
    thumbUrl: payload.snapshot.thumbUrl,
  })
  activateDocument(payload.id)
  markDirty()
}

const handleRemoveDocument = (payload: {
  id: string
  snapshot: PendingSnapshot
}): void => {
  if (documents.value.length <= 1) {
    showError('至少保留一张图片')
    return
  }

  updateDocument(payload.snapshot.documentId, {
    editState: payload.snapshot.editState,
    thumbUrl: payload.snapshot.thumbUrl,
  })

  const index = documents.value.findIndex((doc) => doc.id === payload.id)
  if (index < 0) return

  const [removed] = documents.value.splice(index, 1)
  revokeThumbUrl(removed.thumbUrl)

  if (activeDocumentId.value === payload.id) {
    const next = documents.value[Math.min(index, documents.value.length - 1)]
    if (next) activateDocument(next.id)
  }

  markDirty()
}

const handleAddImage = async (payload: {
  file: File
  snapshot: { documentId: string; editState: ImageEditState; thumbUrl: string }
}): Promise<void> => {
  if (!isImageFile(payload.file)) {
    showError('请选择有效的图片文件')
    return
  }
  try {
    updateDocument(payload.snapshot.documentId, {
      editState: payload.snapshot.editState,
      thumbUrl: payload.snapshot.thumbUrl,
    })
    const id = await addDocumentFromFile(payload.file)
    activateDocument(id)
    markDirty()
  } catch {
    showError('图片加载失败')
  }
}

const handleExport = async (payload: {
  canvas: HTMLCanvasElement
  settings: ExportSettings
}): Promise<void> => {
  try {
    const result = await exportImageWithSettings(payload.canvas, payload.settings)
    if (!result) return

    const count = result.paths.length
    const message = count > 1 ? `已导出 ${count} 个文件` : '图片已导出'
    showSuccess(message, {
      label: '在文件夹中显示',
      run: () => revealExportPath(result.paths[0]),
    })
  } catch (e) {
    showError(formatInvokeError(e))
  }
}

const createSmallPreviewBase64 = async (sourceBase64: string): Promise<string | null> => {
  try {
    const bytes = base64ToBytes(sourceBase64)
    const blob = new Blob([bytes.slice()], { type: 'image/png' })
    const bitmap = await createImageBitmap(blob)
    const edge = Math.max(bitmap.width, bitmap.height, 1)
    const size = 96
    const scale = size / edge
    const cssW = Math.max(1, Math.round(bitmap.width * scale))
    const cssH = Math.max(1, Math.round(bitmap.height * scale))
    const canvas = document.createElement('canvas')
    canvas.width = cssW
    canvas.height = cssH
    const ctx = canvas.getContext('2d')
    if (!ctx) {
      bitmap.close()
      return null
    }
    ctx.drawImage(bitmap, 0, 0, cssW, cssH)
    bitmap.close()
    const out = await new Promise<Blob | null>((resolve) => {
      canvas.toBlob(resolve, 'image/png')
    })
    if (!out) return null
    return uint8ToBase64(new Uint8Array(await out.arrayBuffer()))
  } catch {
    return null
  }
}

const buildProjectPayload = async (
  latestSnapshot?: PendingSnapshot,
  name?: string,
): Promise<ImageEditorProject> => {
  if (latestSnapshot) {
    updateDocument(latestSnapshot.documentId, {
      editState: latestSnapshot.editState,
      thumbUrl: latestSnapshot.thumbUrl,
    })
  }

  const docs = await Promise.all(
    documents.value.map(async (doc) => ({
      id: doc.id,
      name: doc.name,
      width: doc.width,
      height: doc.height,
      exportName: doc.exportName,
      editState: doc.editState,
      sourceBase64: doc.editState?.originalPng ?? await fileToBase64(doc.file),
    })),
  )

  const activeDoc = documents.value.find((doc) => doc.id === activeDocumentId.value)
    ?? documents.value[0]
  const previewSource = activeDoc?.editState?.workingPng
    ?? docs.find((doc) => doc.id === activeDoc?.id)?.sourceBase64
    ?? null
  const previewBase64 = previewSource
    ? await createSmallPreviewBase64(previewSource)
    : null

  const id = projectId.value ?? createProjectId()
  const resolvedName = name ?? projectName.value ?? defaultProjectName()

  return {
    id,
    name: resolvedName,
    updatedAt: Date.now(),
    activeDocumentId: activeDocumentId.value || documents.value[0]?.id || '',
    exportMode: exportMode.value,
    exportFormatId: exportFormatId.value,
    compressionPresetId: compressionPresetId.value,
    sizeScalePresetId: sizeScalePresetId.value,
    thumbnailSizes: [...thumbnailSizes.value],
    icoSizes: [...icoSizes.value],
    previewBase64,
    documents: docs,
  }
}

const performSave = async (
  snapshot: PendingSnapshot,
  name: string,
): Promise<boolean> => {
  if (saving.value) return false
  const safeName = sanitizeProjectName(name)
  if (!safeName) {
    showError('请输入项目名称')
    return false
  }

  saving.value = true
  try {
    const project = await buildProjectPayload(snapshot, safeName)
    const savedId = await saveImageEditorProject(JSON.stringify(project))
    projectId.value = savedId
    projectName.value = safeName
    hasUnsavedChanges.value = false
    await notifyImageEditorProjectsChanged()
    showSuccess('项目已保存，可在入口页继续编辑')
    return true
  } catch (e) {
    showError(formatInvokeError(e))
    return false
  } finally {
    saving.value = false
  }
}

const confirmSaveName = async (): Promise<void> => {
  const snapshot = pendingSnapshot.value
  if (!snapshot) return
  const ok = await performSave(snapshot, saveNameInput.value)
  if (!ok) return

  const shouldExit = saveDialogMode.value === 'save-and-exit'
  closeSaveNameDialog()
  pendingSnapshot.value = null
  if (shouldExit) {
    await closeEditorWindow()
  }
}

const handleSave = (payload: {
  snapshot: PendingSnapshot
}): void => {
  openSaveNameDialog(payload.snapshot, 'save')
}
</script>

<template>
  <div class="image-editor-window">
    <EditorSession
      v-if="ready && activeDocument"
      :key="activeDocumentId"
      :document-id="activeDocumentId"
      :file="activeDocument.file"
      :image-name="activeDocument.name"
      :image-width="activeDocument.width"
      :image-height="activeDocument.height"
      :restored-edit-state="restoredEditState"
      :document-history="documentHistory"
      :active-document-id="activeDocumentId"
      :saving="saving"
      v-model:export-mode="exportMode"
      v-model:export-name="exportName"
      v-model:export-format-id="exportFormatId"
      v-model:compression-preset-id="compressionPresetId"
      v-model:size-scale-preset-id="sizeScalePresetId"
      v-model:thumbnail-sizes="thumbnailSizes"
      v-model:ico-sizes="icoSizes"
      :export-formats="IMAGE_EXPORT_FORMATS"
      :compression-presets="IMAGE_COMPRESSION_PRESETS"
      :size-scale-presets="IMAGE_SIZE_SCALE_PRESETS"
      @state-snapshot="handleStateSnapshot"
      @switch-document="handleSwitchDocument"
      @add-image="handleAddImage"
      @remove-document="handleRemoveDocument"
      @export="handleExport"
      @save="handleSave"
      @exit-request="handleExitRequest"
    />
    <div v-else-if="bootError" class="image-editor-window__loading">
      <p class="image-editor-window__boot-error">{{ bootError }}</p>
      <VoidButton variant="secondary" size="medium" @click="handleBootErrorClose">
        关闭窗口
      </VoidButton>
    </div>
    <div v-else class="image-editor-window__loading">
      <Loader2 :size="32" :stroke-width="2" class="image-editor-window__spin" />
      <p>正在加载图片…</p>
    </div>

    <VoidToast :controller="toast" />

    <Teleport to="body">
      <div
        v-if="exitDialogVisible"
        class="image-editor-dialog"
        @click.self="handleExitCancel"
      >
        <div class="image-editor-dialog__panel" role="dialog" aria-labelledby="exit-dialog-title">
          <h3 id="exit-dialog-title" class="image-editor-dialog__title">退出编辑</h3>
          <p class="image-editor-dialog__desc">是否在退出前保存当前项目？</p>
          <div class="image-editor-dialog__actions">
            <VoidButton variant="accent" size="medium" @click="handleExitSave">
              保存并退出
            </VoidButton>
            <VoidButton variant="secondary" size="medium" @click="handleExitDiscard">
              直接退出
            </VoidButton>
            <VoidButton variant="secondary" size="medium" @click="handleExitCancel">
              取消
            </VoidButton>
          </div>
        </div>
      </div>

      <div
        v-if="saveDialogVisible"
        class="image-editor-dialog"
        @click.self="closeSaveNameDialog"
      >
        <div class="image-editor-dialog__panel" role="dialog" aria-labelledby="save-dialog-title">
          <h3 id="save-dialog-title" class="image-editor-dialog__title">保存项目</h3>
          <label class="image-editor-dialog__field">
            <span>项目名称</span>
            <input
              v-model="saveNameInput"
              type="text"
              class="image-editor-dialog__input"
              placeholder="请输入项目名称"
              @keydown.enter="confirmSaveName"
            />
          </label>
          <div class="image-editor-dialog__actions">
            <VoidButton
              variant="accent"
              size="medium"
              :disabled="saving"
              :loading="saving"
              @click="confirmSaveName"
            >
              {{ saving ? '保存中…' : '保存' }}
            </VoidButton>
            <VoidButton variant="secondary" size="medium" :disabled="saving" @click="closeSaveNameDialog">
              取消
            </VoidButton>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style lang="less" scoped>
.image-editor-window {
  width: 100%;
  height: 100%;
  min-height: 0;
  background: #0a0a0c;

  &__loading {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: #9ca3af;
    font-size: 13px;
    padding: 24px;
    text-align: center;
  }

  &__boot-error {
    margin: 0;
    max-width: 360px;
    line-height: 1.5;
    color: #fca5a5;
  }

  &__spin {
    color: #c084fc;
    animation: spin 0.9s linear infinite;
  }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.image-editor-dialog {
  position: fixed;
  inset: 0;
  z-index: 10003;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: rgba(0, 0, 0, 0.55);

  &__panel {
    width: min(360px, 100%);
    padding: 18px;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: #16171d;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
  }

  &__title {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 600;
    color: #f3f4f6;
  }

  &__desc {
    margin: 0 0 16px;
    font-size: 13px;
    line-height: 1.5;
    color: #9ca3af;
  }

  &__field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 16px;
    font-size: 12px;
    color: #9ca3af;
  }

  &__input {
    height: 36px;
    padding: 0 10px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
    color: #e5e7eb;
    font-size: 14px;
    box-sizing: border-box;

    &:focus {
      outline: none;
      border-color: rgba(192, 132, 252, 0.5);
    }
  }

  &__actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    justify-content: flex-end;
  }
}
</style>
