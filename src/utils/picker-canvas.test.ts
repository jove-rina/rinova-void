/**
 * picker-canvas.test.ts
 */
import { describe, expect, it } from 'vitest'
import {
  decodeBase64ToBytes,
  displayPercentToScale,
  formatDisplayZoomPercent,
  parseDisplayZoomPercent,
  zoomScaleLimits,
} from '@/utils/picker-canvas'

describe('decodeBase64ToBytes', () => {
  it('decodes base64 to bytes', () => {
    const bytes = decodeBase64ToBytes('AQID')
    expect(Array.from(bytes)).toEqual([1, 2, 3])
  })

  it('decodes empty payload', () => {
    expect(decodeBase64ToBytes('').length).toBe(0)
  })
})

describe('display zoom percent', () => {
  it('formats relative to base scale', () => {
    expect(formatDisplayZoomPercent(0.5, 0.5)).toBe('100%')
    expect(formatDisplayZoomPercent(1, 0.5)).toBe('200%')
  })

  it('parses percent input', () => {
    expect(parseDisplayZoomPercent('150%')).toBe(150)
    expect(parseDisplayZoomPercent('bad')).toBeNull()
  })

  it('converts display percent to scale', () => {
    expect(displayPercentToScale(200, 0.4)).toBeCloseTo(0.8)
  })

  it('derives zoom limits from base scale', () => {
    expect(zoomScaleLimits(0.5)).toEqual({ min: 0.05, max: 4 })
  })
})
