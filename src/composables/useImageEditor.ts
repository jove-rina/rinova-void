/**
 * useImageEditor.ts
 * 图片编辑器 — 多图上传、项目、主窗口内编辑会话
 */
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  deleteImageEditorProject,
  listImageEditorProjects,
  loadImageEditorProject,
  type EditorSessionMeta,
} from '@/api/image-editor'
import {
  createProjectId,
  fileToBase64,
  parseProjectJson,
  type ImageEditorProjectSummary,
} from '@/utils/image-editor-project'
import {
  IMAGE_COMPRESSION_PRESETS,
  IMAGE_EXPORT_FORMATS,
  IMAGE_SIZE_SCALE_PRESETS,
  sanitizeExportStem,
} from '@/utils/image-editor-export'
import { createDocumentId, type ImageEditState } from '@/utils/image-editor-document'
import {
  fileFromPath,
  isImageFile,
  isImagePath,
  readImageMeta,
} from '@/utils/image-file-load'
import { base64ToBytes } from '@/utils/image-editor-export'
import { isMacOs } from '@/utils/platform'
import { useToast, type ToastAction } from '@/composables/useToast'

export interface EditorSessionImage {
  id: string
  name: string
  width: number
  height: number
  exportName: string
  bytes: Uint8Array
  editState: ImageEditState | null
}

export interface EditorSessionPayload {
  meta: EditorSessionMeta
  images: EditorSessionImage[]
}

export type { ToastAction }

export interface EntryImage {
  id: string
  file: File
  name: string
  width: number
  height: number
  thumbUrl: string
}

const createEntryThumbUrl = async (file: File): Promise<string> => {
  const bitmap = await createImageBitmap(file)
  const edge = Math.max(bitmap.width, bitmap.height, 1)
  const size = 56
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

export const useImageEditor = () => {
  const entryImages = ref<EntryImage[]>([])
  const activeEntryId = ref('')
  const savedProjects = ref<ImageEditorProjectSummary[]>([])
  const loading = ref(false)
  const dragOver = ref(false)
  const editorSession = ref<EditorSessionPayload | null>(null)
  const toast = useToast()

  const activeEntry = computed(() =>
    entryImages.value.find((item) => item.id === activeEntryId.value) ?? null,
  )

  const previewUrlRef = ref<string | null>(null)

  const syncPreviewUrl = (): void => {
    if (previewUrlRef.value) {
      URL.revokeObjectURL(previewUrlRef.value)
      previewUrlRef.value = null
    }
    const active = activeEntry.value
    if (active) {
      previewUrlRef.value = URL.createObjectURL(active.file)
    }
  }

  const showSuccess = (message: string, action?: ToastAction): void => {
    toast.showSuccess(message, action)
  }

  const showError = (message: string): void => {
    toast.showError(message)
  }

  const revokeEntryThumb = (url: string): void => {
    if (url) URL.revokeObjectURL(url)
  }

  const addEntryFile = async (file: File): Promise<void> => {
    const meta = await readImageMeta(file)
    const thumbUrl = await createEntryThumbUrl(file)
    const id = createDocumentId()
    entryImages.value.push({
      id,
      file: meta.file,
      name: meta.name,
      width: meta.width,
      height: meta.height,
      thumbUrl,
    })
    activeEntryId.value = id
    syncPreviewUrl()
  }

  const handleFilesSelect = async (files: FileList | File[] | null | undefined): Promise<void> => {
    const list = files ? Array.from(files) : []
    const valid = list.filter(isImageFile)
    if (valid.length === 0) {
      showError('请选择有效的图片文件')
      return
    }
    loading.value = true
    try {
      for (const file of valid) {
        await addEntryFile(file)
      }
    } catch {
      showError('图片加载失败')
    } finally {
      loading.value = false
    }
  }

  const handleFileSelect = async (file: File | null | undefined): Promise<void> => {
    if (!file) return
    await handleFilesSelect([file])
  }

  const handlePathSelect = async (path: string): Promise<void> => {
    if (!isImagePath(path)) {
      showError('请拖拽有效的图片文件')
      return
    }
    loading.value = true
    try {
      const file = await fileFromPath(path)
      await addEntryFile(file)
    } catch {
      showError('图片加载失败')
    } finally {
      loading.value = false
      dragOver.value = false
    }
  }

  const selectEntryImage = (id: string): void => {
    if (!entryImages.value.some((item) => item.id === id)) return
    activeEntryId.value = id
    syncPreviewUrl()
  }

  const removeEntryImage = (id: string): void => {
    const index = entryImages.value.findIndex((item) => item.id === id)
    if (index < 0) return
    const [removed] = entryImages.value.splice(index, 1)
    revokeEntryThumb(removed.thumbUrl)
    if (activeEntryId.value === id) {
      const next = entryImages.value[Math.min(index, entryImages.value.length - 1)]
      activeEntryId.value = next?.id ?? ''
      syncPreviewUrl()
    }
  }

  const refreshProjects = async (): Promise<void> => {
    try {
      savedProjects.value = await listImageEditorProjects()
    } catch {
      savedProjects.value = []
    }
  }

  const buildDefaultSessionMeta = (
    activeId: string,
    projectId: string | null = null,
    projectName: string | null = null,
  ): EditorSessionMeta => ({
    activeId,
    exportMode: 'general',
    exportFormatId: IMAGE_EXPORT_FORMATS[0].id,
    compressionPresetId: IMAGE_COMPRESSION_PRESETS[0].id,
    sizeScalePresetId: IMAGE_SIZE_SCALE_PRESETS[0].id,
    thumbnailSizes: [],
    icoSizes: [],
    projectId,
    projectName,
  })

  const exitEditorFullscreen = async (): Promise<void> => {
    try {
      if (!isMacOs()) {
        await getCurrentWindow().setFullscreen(false)
      } else {
        await getCurrentWindow().unmaximize()
      }
    } catch {
      // 非 Tauri 环境
    }
  }

  const openEditorWithImages = (
    images: EditorSessionImage[],
    meta: EditorSessionMeta,
  ): void => {
    if (images.length === 0) {
      throw new Error('请至少添加一张图片')
    }
    editorSession.value = { meta, images }
  }

  const handleExitEditor = async (): Promise<void> => {
    editorSession.value = null
    await exitEditorFullscreen()
    await refreshProjects()
  }

  const handleStartEdit = async (): Promise<void> => {
    if (entryImages.value.length === 0) return
    loading.value = true
    try {
      const activeId = activeEntryId.value || entryImages.value[0].id
      const images = await Promise.all(
        entryImages.value.map(async (item) => ({
          id: item.id,
          name: item.name,
          width: item.width,
          height: item.height,
          exportName: sanitizeExportStem(item.name),
          bytes: new Uint8Array(await item.file.arrayBuffer()),
          editState: null,
        })),
      )
      openEditorWithImages(images, buildDefaultSessionMeta(activeId))
    } catch (e) {
      showError(e instanceof Error ? e.message : '无法进入编辑')
    } finally {
      loading.value = false
    }
  }

  const handleOpenProject = async (projectId: string): Promise<void> => {
    loading.value = true
    try {
      const raw = await loadImageEditorProject(projectId)
      const project = parseProjectJson(raw)
      const images = project.documents.map((doc) => ({
        id: doc.id,
        name: doc.name,
        width: doc.width,
        height: doc.height,
        exportName: doc.exportName,
        bytes: base64ToBytes(doc.sourceBase64),
        editState: doc.editState,
      }))
      openEditorWithImages(images, {
        activeId: project.activeDocumentId,
        exportMode: project.exportMode,
        exportFormatId: project.exportFormatId,
        compressionPresetId: project.compressionPresetId,
        sizeScalePresetId: project.sizeScalePresetId,
        thumbnailSizes: [...project.thumbnailSizes],
        icoSizes: [...project.icoSizes],
        projectId: project.id,
        projectName: project.name,
      })
    } catch (e) {
      showError(e instanceof Error ? e.message : '无法打开项目')
    } finally {
      loading.value = false
    }
  }

  const handleDeleteProject = async (projectId: string): Promise<void> => {
    try {
      await deleteImageEditorProject(projectId)
      await refreshProjects()
      showSuccess('项目已删除')
    } catch (e) {
      showError(e instanceof Error ? e.message : '删除项目失败')
    }
  }

  let unlistenProjectsChanged: UnlistenFn | undefined

  onMounted(async () => {
    await refreshProjects()
    try {
      unlistenProjectsChanged = await listen('image-editor-projects-changed', () => {
        void refreshProjects()
      })
    } catch {
      // 非 Tauri 环境
    }
  })

  onUnmounted(() => {
    void unlistenProjectsChanged?.()
    unlistenProjectsChanged = undefined
    if (previewUrlRef.value) URL.revokeObjectURL(previewUrlRef.value)
    for (const item of entryImages.value) {
      revokeEntryThumb(item.thumbUrl)
    }
    if (editorSession.value) {
      editorSession.value = null
      void exitEditorFullscreen()
    }
  })

  return {
    entryImages,
    activeEntryId,
    activeEntry,
    previewUrl: previewUrlRef,
    savedProjects,
    loading,
    dragOver,
    editorSession,
    toast,
    handleFileSelect,
    handleFilesSelect,
    handlePathSelect,
    selectEntryImage,
    removeEntryImage,
    handleStartEdit,
    handleOpenProject,
    handleDeleteProject,
    handleExitEditor,
    refreshProjects,
    showSuccess,
    showError,
    fileToBase64,
    createProjectId,
  }
}
