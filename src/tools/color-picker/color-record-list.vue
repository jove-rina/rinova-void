<script setup lang="ts">
/**
 * color-record-list.vue
 * 取色记录列表 — 工具页与取色面板共用
 */
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { ChevronDown, Copy, Trash2 } from '@lucide/vue'
import { toHsl, toRgb } from '@/utils/color-format'
import type { ColorRecord } from '@/utils/color-records'

const props = withDefaults(
  defineProps<{
    records: ColorRecord[]
    emptyText?: string
    scrollable?: boolean
  }>(),
  {
    emptyText: '还没有取色记录',
    scrollable: false,
  },
)

const emit = defineEmits<{
  rename: [id: string, name: string]
  'delete-record': [id: string]
  'copy-hex': [record: ColorRecord]
  'copy-rgb': [record: ColorRecord]
  'copy-hsl': [record: ColorRecord]
}>()

const editingId = ref<string | null>(null)
const copyMenuId = ref<string | null>(null)
const nameInputs = ref<Record<string, HTMLInputElement | null>>({})

const setNameInputRef = (id: string, el: HTMLInputElement | null): void => {
  if (el) {
    nameInputs.value[id] = el
  }
}

const startEdit = async (record: ColorRecord): Promise<void> => {
  editingId.value = record.id
  copyMenuId.value = null
  await nextTick()
  nameInputs.value[record.id]?.focus()
  nameInputs.value[record.id]?.select()
}

const finishEdit = (id: string, event: Event): void => {
  const target = event.target as HTMLInputElement
  const record = props.records.find((item) => item.id === id)
  if (record && record.name !== target.value.trim()) {
    emit('rename', id, target.value)
  }
  editingId.value = null
}

const cancelEdit = (id: string): void => {
  if (editingId.value === id) {
    editingId.value = null
  }
}

const toggleCopyMenu = (id: string): void => {
  copyMenuId.value = copyMenuId.value === id ? null : id
}

const closeCopyMenu = (): void => {
  copyMenuId.value = null
}

const onDocumentPointerDown = (event: PointerEvent): void => {
  const target = event.target as HTMLElement | null
  if (target?.closest('.color-record-list__copy-wrap')) return
  closeCopyMenu()
}

const requestDelete = (id: string): void => {
  closeCopyMenu()
  emit('delete-record', id)
}

const copyHex = (record: ColorRecord): void => {
  emit('copy-hex', record)
  closeCopyMenu()
}

const copyRgb = (record: ColorRecord): void => {
  emit('copy-rgb', record)
  closeCopyMenu()
}

const copyHsl = (record: ColorRecord): void => {
  emit('copy-hsl', record)
  closeCopyMenu()
}

onMounted(() => {
  document.addEventListener('pointerdown', onDocumentPointerDown)
})

onUnmounted(() => {
  document.removeEventListener('pointerdown', onDocumentPointerDown)
})
</script>

<template>
  <div
    class="color-record-list"
    :class="{ 'color-record-list--scrollable': scrollable }"
  >
    <p v-if="records.length === 0" class="color-record-list__empty">
      {{ emptyText }}
    </p>

    <ul
      v-else
      class="color-record-list__items"
      :class="{ 'color-record-list__items--scrollable': scrollable }"
    >
      <li v-for="record in records" :key="record.id" class="color-record-list__item">
        <span class="color-record-list__swatch" :style="{ background: record.hex }" />
        <div class="color-record-list__body">
          <input
            v-if="editingId === record.id"
            :ref="(el) => setNameInputRef(record.id, el as HTMLInputElement | null)"
            class="color-record-list__name-input"
            type="text"
            :value="record.name"
            :aria-label="`${record.name} 标题`"
            @blur="finishEdit(record.id, $event)"
            @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
            @keydown.escape="cancelEdit(record.id)"
          />
          <span
            v-else
            class="color-record-list__title"
            title="双击编辑名称"
            @dblclick.stop="startEdit(record)"
          >
            {{ record.name }}
          </span>
          <span class="color-record-list__hex">{{ record.hex }}</span>
          <span class="color-record-list__rgb">{{ toRgb(record.r, record.g, record.b) }}</span>
          <span class="color-record-list__hsl">{{ toHsl(record.r, record.g, record.b) }}</span>
        </div>
        <div class="color-record-list__actions">
          <div class="color-record-list__copy-wrap" @pointerdown.stop @click.stop>
            <button
              type="button"
              class="color-record-list__action color-record-list__copy-trigger"
              aria-label="复制颜色"
              aria-haspopup="menu"
              :aria-expanded="copyMenuId === record.id"
              @click.stop="toggleCopyMenu(record.id)"
            >
              <Copy :size="14" :stroke-width="2" />
              <ChevronDown :size="10" :stroke-width="2" />
            </button>
            <div
              v-if="copyMenuId === record.id"
              class="color-record-list__copy-menu"
              role="menu"
              @pointerdown.stop
              @click.stop
            >
              <button
                type="button"
                class="color-record-list__copy-item"
                role="menuitem"
                @click.stop="copyHex(record)"
              >
                复制 HEX
              </button>
              <button
                type="button"
                class="color-record-list__copy-item"
                role="menuitem"
                @click.stop="copyRgb(record)"
              >
                复制 RGB
              </button>
              <button
                type="button"
                class="color-record-list__copy-item"
                role="menuitem"
                @click.stop="copyHsl(record)"
              >
                复制 HSL
              </button>
            </div>
          </div>
          <button
            type="button"
            class="color-record-list__action color-record-list__action--danger"
            aria-label="删除记录"
            @pointerdown.stop
            @click.stop="requestDelete(record.id)"
          >
            <Trash2 :size="14" :stroke-width="2" />
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>

<style lang="less" scoped>
.color-record-list {
  display: flex;
  flex-direction: column;
  min-height: 0;

  &--scrollable {
    flex: 1;
    min-height: 0;
  }

  &__empty {
    margin: 0;
    font-size: 12px;
    line-height: 1.5;
    color: var(--void-text-dim);
    opacity: 0.85;
  }

  &__items {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;

    &--scrollable {
      flex: 1;
      min-height: 0;
      overflow-y: auto;
    }
  }

  &__item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  &__swatch {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    flex-shrink: 0;
  }

  &__body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  &__title {
    font-size: 13px;
    font-weight: 600;
    color: var(--void-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: text;
    user-select: none;
  }

  &__name-input {
    width: 100%;
    padding: 2px 4px;
    margin: -2px -4px;
    border: 1px solid var(--void-accent-dim);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
    font-size: 13px;
    font-weight: 600;
    color: var(--void-text);
    outline: none;
    user-select: text;
  }

  &__hex {
    font-size: 12px;
    color: var(--void-text);
    font-family: ui-monospace, 'Cascadia Code', monospace;
  }

  &__rgb {
    font-size: 11px;
    color: var(--void-text-dim);
    font-family: ui-monospace, 'Cascadia Code', monospace;
  }

  &__hsl {
    font-size: 11px;
    color: var(--void-text-dim);
    font-family: ui-monospace, 'Cascadia Code', monospace;
    opacity: 0.85;
  }

  &__actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  &__action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--void-text-dim);
    cursor: pointer;

    &:hover {
      background: rgba(255, 255, 255, 0.08);
      color: var(--void-text);
    }

    &--danger:hover {
      background: rgba(239, 68, 68, 0.12);
      color: #fca5a5;
    }
  }

  &__copy-wrap {
    position: relative;
  }

  &__copy-trigger {
    width: auto;
    padding: 0 4px;
    gap: 1px;
  }

  &__copy-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 20;
    min-width: 108px;
    padding: 4px;
    border-radius: 8px;
    background: var(--void-bg);
    border: 1px solid var(--void-border);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }

  &__copy-item {
    display: block;
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--void-text);
    font-size: 12px;
    text-align: left;
    cursor: pointer;

    &:hover {
      background: rgba(255, 255, 255, 0.08);
      color: var(--void-accent);
    }
  }
}
</style>
