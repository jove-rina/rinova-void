/**
 * clash-prefs.ts
 * Clash 工具表单偏好（订阅 URL、端口）的 localStorage 持久化
 *
 * 用户在 Clash 工具页填写的 URL 与端口会写入浏览器本地存储，
 * 下次打开应用时自动恢复，避免重复输入。
 */

/** localStorage 键名，命名空间前缀 void. 避免与其他站点键冲突 */
const STORAGE_KEY = 'void.clash.prefs'

/** 首次使用或存储损坏时的默认监听端口 */
const DEFAULT_PORT = 25500

/**
 * 可持久化的 Clash 表单字段结构。
 */
export interface ClashPrefs {
  /** 远程订阅 URL，允许空字符串 */
  url: string
  /** HTTP 服务监听端口，合法范围 1024–65535 */
  port: number
}

/**
 * 从 localStorage 读取 Clash 偏好设置。
 *
 * 读取策略：
 * - 无数据 → 返回空 URL + 默认端口
 * - JSON 解析失败 → 同上，静默降级
 * - port 非数字或超出范围 → 回退 DEFAULT_PORT
 * - url 非字符串 → 回退空字符串
 *
 * @returns 校验后的偏好对象，保证 port 始终在合法区间
 */
export const loadClashPrefs = (): ClashPrefs => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { url: '', port: DEFAULT_PORT }

    const parsed = JSON.parse(raw) as Partial<ClashPrefs>
    const port =
      typeof parsed.port === 'number' && parsed.port >= 1024 && parsed.port <= 65535
        ? parsed.port
        : DEFAULT_PORT

    return {
      url: typeof parsed.url === 'string' ? parsed.url : '',
      port,
    }
  } catch {
    return { url: '', port: DEFAULT_PORT }
  }
}

/**
 * 将 Clash 偏好写入 localStorage。
 *
 * 写入前会对 url 做 trim，避免首尾空格导致订阅请求异常。
 * 隐私模式或配额满时写入可能失败，此时静默忽略（不阻断用户操作）。
 *
 * @param prefs - 待保存的 URL 与端口
 */
export const saveClashPrefs = (prefs: ClashPrefs): void => {
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        url: prefs.url.trim(),
        port: prefs.port,
      }),
    )
  } catch {
    // 隐私模式 / 存储配额已满 — 忽略，不影响核心功能
  }
}
