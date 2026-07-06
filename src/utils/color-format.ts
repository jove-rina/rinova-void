/**
 * color-format.ts
 * RGB / HEX / HSL 颜色格式转换
 */
/**
 * 将 RGB 分量格式化为小写 HEX 字符串。
 */
export const toHex = (r: number, g: number, b: number): string =>
  `#${[r, g, b].map((v) => v.toString(16).padStart(2, '0')).join('')}`

/** 将 RGB 分量转为 CSS rgb() 字符串。 */
export const toRgb = (r: number, g: number, b: number): string =>
  `rgb(${r}, ${g}, ${b})`

/** 将 RGB 分量转为 CSS hsl() 字符串。 */
export const toHsl = (r: number, g: number, b: number): string => {
  const rn = r / 255
  const gn = g / 255
  const bn = b / 255
  const max = Math.max(rn, gn, bn)
  const min = Math.min(rn, gn, bn)
  const lightness = (max + min) / 2
  const lPct = Math.round(lightness * 100)

  if (max === min) {
    return `hsl(0, 0%, ${lPct}%)`
  }

  const delta = max - min
  const saturation = lightness > 0.5
    ? delta / (2 - max - min)
    : delta / (max + min)
  const sPct = Math.round(saturation * 100)

  let hue = 0
  if (max === rn) {
    hue = ((gn - bn) / delta + (gn < bn ? 6 : 0)) / 6
  } else if (max === gn) {
    hue = ((bn - rn) / delta + 2) / 6
  } else {
    hue = ((rn - gn) / delta + 4) / 6
  }

  return `hsl(${Math.round(hue * 360)}, ${sPct}%, ${lPct}%)`
}

/** 解析 #rrggbb 为 RGB 分量；无效时返回 null */
export const parseHex = (hex: string): [number, number, number] | null => {
  const match = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex)
  if (!match) return null
  return [parseInt(match[1], 16), parseInt(match[2], 16), parseInt(match[3], 16)]
}
