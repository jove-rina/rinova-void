/**
 * clash-prefs.test.ts
 * clash-prefs 模块单元测试
 *
 * 覆盖：默认值、读写往返、URL trim、非法端口回退。
 */
import { describe, expect, it, beforeEach } from 'vitest'
import { loadClashPrefs, saveClashPrefs } from './clash-prefs'

/** 与 clash-prefs.ts 中 STORAGE_KEY 保持一致，用于断言 localStorage 内容 */
const STORAGE_KEY = 'void.clash.prefs'

describe('clash-prefs', () => {
  /** 每个用例前清空 localStorage，保证测试隔离 */
  beforeEach(() => {
    localStorage.clear()
  })

  it('returns defaults when storage is empty', () => {
    expect(loadClashPrefs()).toEqual({ url: '', port: 25500 })
  })

  it('round-trips url and port', () => {
    saveClashPrefs({ url: 'https://sub.example/x', port: 25501 })
    expect(loadClashPrefs()).toEqual({
      url: 'https://sub.example/x',
      port: 25501,
    })
    expect(localStorage.getItem(STORAGE_KEY)).toBeTruthy()
  })

  it('trims url on save', () => {
    saveClashPrefs({ url: '  https://a.com  ', port: 25500 })
    expect(loadClashPrefs().url).toBe('https://a.com')
  })

  it('falls back to default port for invalid stored port', () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ url: 'x', port: 80 }))
    expect(loadClashPrefs().port).toBe(25500)
  })
})
