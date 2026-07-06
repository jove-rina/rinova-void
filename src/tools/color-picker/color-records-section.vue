<script setup lang="ts">
/**
 * color-records-section.vue
 * 取色记录区块 — 工具页与取色操作面板共用
 */
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { ChevronDown, ChevronUp, Download } from '@lucide/vue'
import { MAX_COLOR_RECORDS, type ColorRecord, type ColorRecordExportFormat } from '@/utils/color-records'
import ColorRecordList from './color-record-list.vue'

const props = withDefaults(
  defineProps<{
    records: ColorRecord[]
    emptyText?: string
    collapsible?: boolean
    fill?: boolean
  }>(),
  {
    emptyText: '还没有取色记录',
    collapsible: true,
    fill: false,
  },
)

const expanded = defineModel<boolean>('expanded', { default: false })

const emit = defineEmits<{
  rename: [id: string, name: string]
  'delete-record': [id: string]
  'copy-hex': [record: ColorRecord]
  'copy-rgb': [record: ColorRecord]
  'copy-hsl': [record: ColorRecord]
  export: [format: ColorRecordExportFormat]
}>()

const exportMenuOpen = ref(false)

const listVisible = computed(
  () => props.records.length === 0 || !props.collapsible || expanded.value,
)

const listScrollable = computed(
  () => props.records.length > 0 && (!props.collapsible || expanded.value),
)

const sectionClass = computed(() => ({
  'color-records-section--fill': props.fill,
  'color-records-section--expanded': props.fill && listScrollable.value,
}))

const toggleExpanded = (): void => {
  expanded.value = !expanded.value
}

const closeExportMenu = (): void => {
  exportMenuOpen.value = false
}

const toggleExportMenu = (): void => {
  exportMenuOpen.value = !exportMenuOpen.value
}

const onExport = (format: ColorRecordExportFormat): void => {
  emit('export', format)
  closeExportMenu()
}

const onDocumentPointerDown = (event: PointerEvent): void => {
  const target = event.target as HTMLElement | null
  if (target?.closest('.color-records-section__export-wrap')) return
  closeExportMenu()
}

onMounted(() => {
  document.addEventListener('pointerdown', onDocumentPointerDown)
})

onUnmounted(() => {
  document.removeEventListener('pointerdown', onDocumentPointerDown)
})
</script>

<template>
  <section class="color-records-section" :class="sectionClass">
    <div class="color-records-section__head">
      <div class="color-records-section__summary">
        <h3 class="color-records-section__title">取色记录</h3>
        <span class="color-records-section__count">
          共 {{ records.length }} 条
          <span v-if="records.length >= MAX_COLOR_RECORDS" class="color-records-section__limit">
            （已达上限 {{ MAX_COLOR_RECORDS }}）
          </span>
        </span>
      </div>
      <div v-if="records.length > 0" class="color-records-section__actions">
        <div class="color-records-section__export-wrap" @pointerdown.stop @click.stop>
          <button
            type="button"
            class="color-records-section__export"
            aria-label="导出记录"
            aria-haspopup="menu"
            :aria-expanded="exportMenuOpen"
            @click.stop="toggleExportMenu"
          >
            <Download :size="14" :stroke-width="2" />
            导出
            <ChevronDown :size="10" :stroke-width="2" />
          </button>
          <div
            v-if="exportMenuOpen"
            class="color-records-section__export-menu"
            role="menu"
            @pointerdown.stop
            @click.stop
          >
            <button
              type="button"
              class="color-records-section__export-item"
              role="menuitem"
              @click.stop="onExport('json')"
            >
              JSON
            </button>
            <button
              type="button"
              class="color-records-section__export-item"
              role="menuitem"
              @click.stop="onExport('markdown')"
            >
              Markdown
            </button>
            <button
              type="button"
              class="color-records-section__export-item"
              role="menuitem"
              @click.stop="onExport('csv')"
            >
              CSV
            </button>
          </div>
        </div>
        <button
          v-if="collapsible"
          type="button"
          class="color-records-section__toggle"
          :aria-expanded="expanded"
          @click="toggleExpanded"
        >
          {{ expanded ? '收起' : '展开' }}
          <ChevronUp v-if="expanded" :size="14" :stroke-width="2" />
          <ChevronDown v-else :size="14" :stroke-width="2" />
        </button>
      </div>
    </div>

    <ColorRecordList
      v-if="listVisible"
      :records="records"
      :scrollable="listScrollable"
      :empty-text="emptyText"
      @rename="(id, name) => emit('rename', id, name)"
      @delete-record="(id) => emit('delete-record', id)"
      @copy-hex="(record) => emit('copy-hex', record)"
      @copy-rgb="(record) => emit('copy-rgb', record)"
      @copy-hsl="(record) => emit('copy-hsl', record)"
    />
  </section>
</template>

<style lang="less" scoped>
.color-records-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--void-border);
  flex-shrink: 0;

  &--fill {
    flex: 1;
    min-height: 0;
  }

  &--expanded {
    display: flex;
    flex-direction: column;
  }

  &__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-shrink: 0;
  }

  &__summary {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
  }

  &__title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--void-text);
    flex-shrink: 0;
  }

  &__count {
    font-size: 12px;
    color: var(--void-text-dim);
  }

  &__limit {
    color: #fbbf24;
  }

  &__actions {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  &__export-wrap {
    position: relative;
  }

  &__export {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border: none;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--void-text-dim);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;

    &:hover {
      background: rgba(255, 255, 255, 0.1);
      color: var(--void-text);
    }
  }

  &__export-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 30;
    min-width: 120px;
    padding: 4px;
    border-radius: 8px;
    background: var(--void-bg);
    border: 1px solid var(--void-border);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }

  &__export-item {
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

  &__toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border: none;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--void-accent);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    flex-shrink: 0;

    &:hover {
      background: rgba(255, 255, 255, 0.1);
    }
  }
}
</style>
