/**
 * picker-canvas.test.ts
 */
import { describe, expect, it } from 'vitest'
import { decodeBase64ToBytes } from '@/utils/picker-canvas'

describe('decodeBase64ToBytes', () => {
  it('decodes base64 to bytes', () => {
    const bytes = decodeBase64ToBytes('AQID')
    expect(Array.from(bytes)).toEqual([1, 2, 3])
  })

  it('decodes empty payload', () => {
    expect(decodeBase64ToBytes('').length).toBe(0)
  })
})
