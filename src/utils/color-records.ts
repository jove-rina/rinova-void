/**
 * color-records.ts
 * 取色记录 localStorage 持久化
 */
import { toHex, toHsl, toRgb } from '@/utils/color-format'

const STORAGE_KEY = 'void.color.records'

/** 最多保留条数（超出时丢弃最旧记录） */
export const MAX_COLOR_RECORDS = 1000

export type ColorRecordExportFormat = 'json' | 'markdown' | 'csv'

export interface ColorRecord {
  id: string
  name: string
  hex: string
  r: number
  g: number
  b: number
  createdAt: number
}

const isValidRecord = (value: unknown): value is ColorRecord => {
  if (!value || typeof value !== 'object') return false
  const item = value as Partial<ColorRecord>
  return (
    typeof item.id === 'string'
    && typeof item.name === 'string'
    && typeof item.hex === 'string'
    && typeof item.r === 'number'
    && typeof item.g === 'number'
    && typeof item.b === 'number'
    && typeof item.createdAt === 'number'
    && item.r >= 0
    && item.r <= 255
    && item.g >= 0
    && item.g <= 255
    && item.b >= 0
    && item.b <= 255
  )
}

export const normalizeHex = (hex: string): string => hex.trim().toLowerCase()

export const findRecordByHex = (
  records: ColorRecord[],
  hex: string,
): ColorRecord | undefined =>
  records.find((record) => normalizeHex(record.hex) === normalizeHex(hex))

export const trimColorRecords = (records: ColorRecord[]): ColorRecord[] =>
  records.slice(0, MAX_COLOR_RECORDS)

export const nextDefaultColorName = (records: ColorRecord[]): string => {
  const used = new Set(records.map((r) => r.name))
  let n = records.length + 1
  while (used.has(`颜色 ${n}`)) n++
  return `颜色 ${n}`
}

export const loadColorRecords = (): ColorRecord[] => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return []

    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []

    return trimColorRecords(
      parsed
        .filter(isValidRecord)
        .sort((a, b) => b.createdAt - a.createdAt),
    )
  } catch {
    return []
  }
}

export const saveColorRecords = (records: ColorRecord[]): void => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(trimColorRecords(records)))
  } catch {
    // 隐私模式 / 配额满 — 忽略
  }
}

export const createColorRecord = (
  rgb: [number, number, number],
  records: ColorRecord[],
): ColorRecord => {
  const [r, g, b] = rgb
  return {
    id: crypto.randomUUID(),
    name: nextDefaultColorName(records),
    hex: toHex(r, g, b),
    r,
    g,
    b,
    createdAt: Date.now(),
  }
}

const escapeCsvCell = (value: string): string => {
  if (/[",\n\r]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`
  }
  return value
}

const exportStamp = (): string => new Date().toISOString().slice(0, 10)

export const serializeColorRecords = (
  records: ColorRecord[],
  format: ColorRecordExportFormat,
): { content: string; filename: string; mime: string } => {
  const stamp = exportStamp()

  if (format === 'json') {
    return {
      content: JSON.stringify(records, null, 2),
      filename: `void-colors-${stamp}.json`,
      mime: 'application/json',
    }
  }

  if (format === 'markdown') {
    const lines = [
      '# Void 取色记录',
      '',
      '| 名称 | HEX | RGB | HSL |',
      '| ---- | --- | --- | --- |',
      ...records.map((record) => {
        const rgb = toRgb(record.r, record.g, record.b)
        const hsl = toHsl(record.r, record.g, record.b)
        const name = record.name.replace(/\|/g, '\\|')
        return `| ${name} | \`${record.hex}\` | ${rgb} | ${hsl} |`
      }),
    ]
    return {
      content: lines.join('\n'),
      filename: `void-colors-${stamp}.md`,
      mime: 'text/markdown',
    }
  }

  const header = 'name,hex,r,g,b,createdAt'
  const rows = records.map((record) => [
    escapeCsvCell(record.name),
    escapeCsvCell(record.hex),
    record.r,
    record.g,
    record.b,
    record.createdAt,
  ].join(','))
  return {
    content: [header, ...rows].join('\n'),
    filename: `void-colors-${stamp}.csv`,
    mime: 'text/csv',
  }
}

const downloadTextFile = (content: string, filename: string, mime: string): void => {
  const blob = new Blob([content], { type: `${mime};charset=utf-8` })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = filename
  anchor.click()
  URL.revokeObjectURL(url)
}

export const exportColorRecords = async (
  records: ColorRecord[],
  format: ColorRecordExportFormat,
): Promise<string | null> => {
  const { content, filename, mime } = serializeColorRecords(records, format)
  try {
    const { exportTextToDownloads } = await import('@/api/export')
    return await exportTextToDownloads(filename, content)
  } catch {
    downloadTextFile(content, filename, mime)
    return null
  }
}
