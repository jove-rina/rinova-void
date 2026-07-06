/**
 * color-format.test.ts
 * 颜色格式转换单元测试
 */
import { describe, expect, it } from 'vitest'
import { parseHex, toHex, toHsl, toRgb } from './color-format'

describe('color-format', () => {
  it('toHex pads single-digit channels', () => {
    expect(toHex(0, 0, 0)).toBe('#000000')
    expect(toHex(255, 255, 255)).toBe('#ffffff')
    expect(toHex(192, 132, 252)).toBe('#c084fc')
  })

  it('toRgb formats css string', () => {
    expect(toRgb(192, 132, 252)).toBe('rgb(192, 132, 252)')
  })

  it('parseHex round-trips valid hex', () => {
    expect(parseHex('#c084fc')).toEqual([192, 132, 252])
    expect(parseHex('invalid')).toBeNull()
  })

  it('toHsl formats css string', () => {
    expect(toHsl(192, 132, 252)).toBe('hsl(270, 95%, 75%)')
    expect(toHsl(0, 0, 0)).toBe('hsl(0, 0%, 0%)')
    expect(toHsl(255, 255, 255)).toBe('hsl(0, 0%, 100%)')
  })
})
