/**
 * color-records.test.ts
 */
import { describe, expect, it } from 'vitest'
import {
  findRecordByHex,
  MAX_COLOR_RECORDS,
  serializeColorRecords,
  trimColorRecords,
  type ColorRecord,
} from './color-records'

const sample = (hex: string, id = '1'): ColorRecord => ({
  id,
  name: '颜色 1',
  hex,
  r: 192,
  g: 132,
  b: 252,
  createdAt: 1,
})

describe('color-records', () => {
  it('findRecordByHex matches case-insensitively', () => {
    const records = [sample('#C084FC')]
    expect(findRecordByHex(records, '#c084fc')?.id).toBe('1')
    expect(findRecordByHex(records, '#000000')).toBeUndefined()
  })

  it('trimColorRecords keeps newest entries only', () => {
    const records = Array.from({ length: MAX_COLOR_RECORDS + 5 }, (_, index) =>
      sample(`#${index.toString(16).padStart(6, '0')}`, String(index)),
    )
    const trimmed = trimColorRecords(records)
    expect(trimmed).toHaveLength(MAX_COLOR_RECORDS)
    expect(trimmed[0]?.id).toBe('0')
    expect(trimmed.at(-1)?.id).toBe(String(MAX_COLOR_RECORDS - 1))
  })

  it('serializeColorRecords supports json, markdown, and csv', () => {
    const records = [sample('#c084fc')]
    const json = serializeColorRecords(records, 'json')
    expect(json.filename.endsWith('.json')).toBe(true)
    expect(json.content).toContain('"hex": "#c084fc"')

    const markdown = serializeColorRecords(records, 'markdown')
    expect(markdown.filename.endsWith('.md')).toBe(true)
    expect(markdown.content).toContain('| 名称 | HEX | RGB | HSL |')
    expect(markdown.content).toContain('`#c084fc`')

    const csv = serializeColorRecords(records, 'csv')
    expect(csv.filename.endsWith('.csv')).toBe(true)
    expect(csv.content).toBe('name,hex,r,g,b,createdAt\n颜色 1,#c084fc,192,132,252,1')
  })
})
