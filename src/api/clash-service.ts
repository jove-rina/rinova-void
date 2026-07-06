/**
 * clash-service.ts
 * Clash 订阅服务的前端 IPC 封装层
 *
 * 本模块通过 Tauri `invoke` 与 Rust 后端通信，对外暴露类型安全的 Promise API。
 * 所有函数均为纯 IPC 转发，不包含业务逻辑；状态机与 UI 交互见 `useClashService`。
 */
import { invoke } from '@tauri-apps/api/core'

/**
 * 后端返回的服务运行状态快照。
 * 由 `get_service_status` 命令产生，用于同步 UI 与真实进程状态。
 */
export interface ServiceStatus {
  /** 进程是否存活：running = 子进程在监听端口；stopped = 未启动或已退出 */
  status: 'running' | 'stopped'
  /** 当前监听端口；stopped 时为 null */
  port: number | null
  /** 原始订阅 URL（用户输入）；stopped 时可能为 null */
  url: string | null
  /** 对外可访问的 HTTP 基址，如 `http://127.0.0.1:25500` */
  base_url: string | null
  /**
   * 实际运行方式：
   * - builtin：Rust SDK 内嵌代理（仅 running 时返回）
   */
  runner: 'builtin' | ''
}

/**
 * 手动刷新订阅的结果。
 * 对应后端 `refresh_service` 命令的响应体。
 */
export interface RefreshResult {
  /** 刷新是否成功完成 */
  ok: boolean
  /** 为 true 表示上一次刷新任务仍在进行，本次请求被跳过 */
  skipped?: boolean
  /** 解析到的 Clash 节点数量（刷新成功时返回） */
  nodes?: number
}

/**
 * 启动服务成功后的元信息。
 * 端口可能与用户请求不同（被占用时自动换端口或回收遗留进程）。
 */
export interface StartServiceResult {
  /** 最终可用的 HTTP 基址 */
  base_url: string
  /** 实际绑定端口 */
  port: number
  /** 用户最初请求的端口 */
  requested_port: number
  /** 是否因冲突改用了其他端口 */
  port_changed: boolean
  /** 是否释放了上次未正常退出的 Void 遗留进程 */
  port_reclaimed: boolean
}

/**
 * 端口占用探测结果。
 * 用于启动前提示用户：可释放、可换端口，或需手动关闭外部程序。
 */
export interface PortCheckResult {
  /** 端口当前是否空闲（无任何监听） */
  available: boolean
  /** 被检测的端口号 */
  port: number
  /** 占用方是否为上次未退出的本应用服务（可一键 reclaim） */
  reclaimable: boolean
  /** 占用方是否为外部程序 */
  foreign: boolean
  /** 附近第一个可用端口；无可用时为 null */
  suggested_port: number | null
}

/**
 * 查询 Clash 订阅服务当前运行状态。
 *
 * @returns 服务状态快照，包含端口、URL、runner 等信息
 */
export const getServiceStatus = (): Promise<ServiceStatus> => {
  return invoke<ServiceStatus>('get_service_status')
}

/**
 * 启动 Clash 订阅 HTTP 服务。
 *
 * @param url - 远程订阅源 URL
 * @param port - 期望监听端口（1024–65535）
 * @param allowFallback - 端口被外部占用时是否自动改用 suggested_port
 * @returns 启动结果，含最终 base_url 与端口变更说明
 */
export const startService = (
  url: string,
  port: number,
  allowFallback = false,
): Promise<StartServiceResult> => {
  return invoke<StartServiceResult>('start_service', { url, port, allowFallback })
}

/**
 * 检测指定端口是否可用，并区分占用来源。
 *
 * @param port - 待检测端口
 */
export const checkPort = (port: number): Promise<PortCheckResult> => {
  return invoke<PortCheckResult>('check_port', { port })
}

/**
 * 强制释放被本应用遗留进程占用的端口。
 * 仅当 `checkPort` 返回 reclaimable=true 时应在 UI 中提供此操作。
 *
 * @param port - 待释放端口
 */
export const reclaimPort = (port: number): Promise<void> => {
  return invoke('reclaim_port', { port })
}

/**
 * 停止正在运行的 Clash 订阅服务子进程。
 *
 * @returns 后端返回的提示信息字符串
 */
export const stopService = (): Promise<string> => {
  return invoke<string>('stop_service')
}

/**
 * 立即从远程订阅源拉取并更新本地缓存的 clash.yaml。
 * 服务运行期间由后端定时任务每 60 分钟自动执行；此为手动触发入口。
 */
export const refreshService = (): Promise<RefreshResult> => {
  return invoke<RefreshResult>('refresh_service')
}

/**
 * 初始化 Tauri 窗口（尺寸、圆角、透明等）。
 * 应在应用根组件挂载时调用一次；浏览器预览模式下 invoke 会失败，可安全忽略。
 */
export const initWindow = (): Promise<void> => {
  return invoke('init_window')
}
