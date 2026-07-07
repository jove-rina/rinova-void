/**
 * image-editor-crop.test.ts
 */
import { describe, expect, it } from 'vitest'
import { normalizeSelection } from '@/utils/image-editor-crop'

describe('normalizeSelection', () => {
  it('creates a free rectangle', () => {
    const sel = normalizeSelection(
      { x: 10, y: 20 },
      { x: 110, y: 80 },
      'rect',
      200,
      200,
    )
    expect(sel).toEqual({ x: 10, y: 20, width: 100, height: 60 })
  })

  it('creates a square from drag', () => {
    const sel = normalizeSelection(
      { x: 0, y: 0 },
      { x: 80, y: 40 },
      'square',
      200,
      200,
    )
    expect(sel).toEqual({ x: 0, y: 0, width: 80, height: 80 })
  })

  it('creates a circle bounding box as square', () => {
    const sel = normalizeSelection(
      { x: 5, y: 5 },
      { x: 55, y: 25 },
      'circle',
      200,
      200,
    )
    expect(sel).toEqual({ x: 5, y: 5, width: 50, height: 50 })
  })

  it('clamps to canvas bounds', () => {
    const sel = normalizeSelection(
      { x: 180, y: 180 },
      { x: 250, y: 250 },
      'rect',
      200,
      200,
    )
    expect(sel).toEqual({ x: 180, y: 180, width: 20, height: 20 })
  })

  it('returns null for tiny drag', () => {
    expect(
      normalizeSelection({ x: 1, y: 1 }, { x: 1, y: 1 }, 'rect', 100, 100),
    ).toBeNull()
  })
})
