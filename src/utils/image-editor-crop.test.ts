/**
 * image-editor-crop.test.ts
 */
import { describe, expect, it } from 'vitest'
import { isPointInSelection, normalizeSelection, resizeSelection, translateSelection } from '@/utils/image-editor-crop'

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

describe('isPointInSelection', () => {
  const sel = { x: 10, y: 20, width: 50, height: 40 }

  it('returns true inside selection', () => {
    expect(isPointInSelection({ x: 30, y: 30 }, sel)).toBe(true)
  })

  it('returns false outside selection', () => {
    expect(isPointInSelection({ x: 5, y: 30 }, sel)).toBe(false)
  })
})

describe('translateSelection', () => {
  it('moves selection within canvas bounds', () => {
    const moved = translateSelection(
      { x: 10, y: 10, width: 40, height: 30 },
      20,
      15,
      200,
      200,
    )
    expect(moved).toEqual({ x: 30, y: 25, width: 40, height: 30 })
  })

  it('clamps selection at canvas edge', () => {
    const moved = translateSelection(
      { x: 150, y: 150, width: 40, height: 30 },
      100,
      100,
      200,
      200,
    )
    expect(moved).toEqual({ x: 160, y: 170, width: 40, height: 30 })
  })
})

describe('resizeSelection', () => {
  it('resizes rectangle from south-east handle', () => {
    const resized = resizeSelection(
      { x: 10, y: 20, width: 40, height: 30 },
      'se',
      20,
      10,
      'rect',
      200,
      200,
    )
    expect(resized).toEqual({ x: 10, y: 20, width: 60, height: 40 })
  })

  it('resizes square from north-west handle', () => {
    const resized = resizeSelection(
      { x: 40, y: 40, width: 60, height: 60 },
      'nw',
      -20,
      -20,
      'square',
      200,
      200,
    )
    expect(resized).toEqual({ x: 20, y: 20, width: 80, height: 80 })
  })

  it('keeps circle bounding box square when resizing east handle', () => {
    const resized = resizeSelection(
      { x: 10, y: 10, width: 40, height: 40 },
      'e',
      20,
      0,
      'circle',
      200,
      200,
    )
    expect(resized).toEqual({ x: 10, y: 10, width: 60, height: 60 })
  })

  it('clamps resized selection to canvas bounds', () => {
    const resized = resizeSelection(
      { x: 150, y: 150, width: 40, height: 40 },
      'se',
      80,
      80,
      'rect',
      200,
      200,
    )
    expect(resized).toEqual({ x: 150, y: 150, width: 50, height: 50 })
  })
})
