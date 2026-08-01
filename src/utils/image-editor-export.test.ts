/**
 * image-editor-export.test.ts
 */
import { describe, expect, it, vi } from 'vitest'
import {
  buildGeneralExportFilename,
  buildSizedExportFilename,
  sanitizeExportStem,
} from '@/utils/image-editor-export'

describe('sanitizeExportStem', () => {
  it('strips path and extension', () => {
    expect(sanitizeExportStem('/tmp/photo.png')).toBe('photo')
  })
})

describe('buildGeneralExportFilename', () => {
  it('uses name and timestamp without type segment', () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-07-08T12:34:56'))
    expect(buildGeneralExportFilename('my-image', 'png')).toBe('my-image_20260708123456.png')
    vi.useRealTimers()
  })
})

describe('buildSizedExportFilename', () => {
  it('uses size as type without timestamp', () => {
    expect(buildSizedExportFilename('avatar', 64, 'png')).toBe('avatar_64x64.png')
    expect(buildSizedExportFilename('avatar', 32, 'ico')).toBe('avatar_32x32.ico')
  })
})

describe('ICO_SIZE_OPTIONS', () => {
  it('does not exceed ICO encoder limit', async () => {
    const { ICO_MAX_SIZE, ICO_SIZE_OPTIONS } = await import('@/utils/image-editor-export')
    expect(Math.max(...ICO_SIZE_OPTIONS)).toBeLessThanOrEqual(ICO_MAX_SIZE)
  })
})
