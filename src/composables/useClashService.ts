/**
 * useClashService.ts
 * Clash 订阅服务的 Vue Composable：状态机、表单持久化与 IPC 编排
 *
 * 职责划分：
 * - 本 composable：UI 状态（idle/starting/running/...）、表单校验、用户操作处理
 * - api/clash-service.ts：纯 Tauri invoke 封装
 * - utils/clash-prefs.ts：localStorage 读写
 *
 * 使用方式：在 Clash 工具页调用 `useClashService()`，解构返回的 ref 与方法绑定到模板。
 */
import { ref, watch } from 'vue'
import {
  checkPort,
  getServiceStatus,
  reclaimPort,
  refreshService,
  startService,
  stopService,
  type ServiceStatus,
} from '@/api/clash-service'
import { loadClashPrefs, saveClashPrefs } from '@/utils/clash-prefs'
import { useToast } from '@/composables/useToast'

/**
 * Clash 工具页面向用户展示的 UI 状态。
 * 与后端 ServiceStatus 不同：包含过渡态（starting/stopping）与前端 error 态。
 */
export type ClashUiStatus = 'idle' | 'starting' | 'running' | 'stopping' | 'error'

/**
 * Clash 订阅服务的状态与操作方法集合。
 *
 * @returns 响应式状态 ref 及事件处理函数，供 Clash 工具组件直接使用
 */
export const useClashService = () => {
  // ── 表单字段：从 localStorage 恢复上次输入 ──────────────────────────
  const saved = loadClashPrefs()
  const url = ref(saved.url)
  const port = ref(saved.port)

  // ── UI 状态 ────────────────────────────────────────────────────────
  const status = ref<ClashUiStatus>('idle')
  const serviceUrl = ref('')
  const toast = useToast()
  const copied = ref(false)
  const refreshing = ref(false)
  const runner = ref<'builtin' | null>(null)

  // ── 端口相关提示与选项 ─────────────────────────────────────────────
  const portHint = ref('')
  const allowPortFallback = ref(false)
  const portReclaimable = ref(false)

  /** 端口输入防抖检测的定时器句柄 */
  let portCheckTimer: ReturnType<typeof setTimeout> | undefined

  /**
   * 将当前 url/port 写入 localStorage。
   * 在 watch 与用户操作（启动成功、状态同步）时调用。
   */
  const persistPrefs = (): void => {
    saveClashPrefs({ url: url.value, port: port.value })
  }

  // url 或 port 变化时自动持久化
  watch([url, port], persistPrefs)

  /**
   * 根据后端端口探测结果更新 portHint 与 portReclaimable。
   *
   * 仅在空闲态且端口合法时检测；运行/启停过程中跳过，避免干扰用户。
   */
  const refreshPortHint = async (): Promise<void> => {
    if (status.value === 'running' || status.value === 'starting' || status.value === 'stopping') {
      portHint.value = ''
      return
    }
    if (!port.value || Number.isNaN(port.value) || port.value < 1024) {
      portHint.value = ''
      return
    }
    try {
      const result = await checkPort(port.value)
      portReclaimable.value = result.reclaimable
      if (result.reclaimable) {
        portHint.value = `端口 ${result.port} 被占用（疑似上次未退出的 Void 服务），启动时将自动释放并继续使用该端口`
      } else if (result.foreign) {
        const alt = result.suggested_port
          ? allowPortFallback.value
            ? `，勾选「自动换端口」后将使用 ${result.suggested_port}`
            : `。可勾选「自动换端口」或关闭占用程序`
          : '，且附近无可用端口'
        portHint.value = `端口 ${result.port} 被其他程序占用${alt}`
      } else {
        portHint.value = ''
      }
    } catch {
      portHint.value = ''
      portReclaimable.value = false
    }
  }

  // 端口输入防抖 300ms 后触发检测，减少 IPC 调用频率
  watch(port, () => {
    if (portCheckTimer) clearTimeout(portCheckTimer)
    portCheckTimer = setTimeout(refreshPortHint, 300)
  })

  // 勾选「自动换端口」后重新计算提示文案
  watch(allowPortFallback, refreshPortHint)

  /**
   * 显示临时成功提示。
   *
   * @param msg - 展示给用户的成功文案
   */
  const showSuccess = (msg: string): void => {
    toast.showSuccess(msg)
  }

  /**
   * 将后端 ServiceStatus 映射到前端 UI 状态与表单字段。
   *
   * @param s - `getServiceStatus` 返回的快照
   */
  const applyServiceStatus = (s: ServiceStatus): void => {
    if (s.status === 'running' && s.base_url) {
      status.value = 'running'
      serviceUrl.value = s.base_url
      runner.value = s.runner === 'builtin' ? 'builtin' : null
      if (s.port) port.value = s.port
      if (s.url) url.value = s.url
      persistPrefs()
    } else {
      status.value = 'idle'
      serviceUrl.value = ''
      runner.value = null
    }
  }

  /**
   * 从后端拉取真实服务状态并同步到 UI。
   * 组件 onMounted 时调用；Tauri 不可用时（浏览器预览）静默失败。
   */
  const syncStatus = async (): Promise<void> => {
    try {
      applyServiceStatus(await getServiceStatus())
      await refreshPortHint()
    } catch {
      // Tauri 不可用（浏览器预览模式）
    }
  }

  /**
   * 校验表单并启动 Clash 订阅服务。
   * 处理端口回收、自动换端口等后端返回的特殊情况并给出对应提示。
   */
  const handleStart = async (): Promise<void> => {
    if (!url.value.trim()) {
      toast.showError('请输入订阅 URL')
      return
    }
    if (!port.value || Number.isNaN(port.value) || port.value < 1024) {
      toast.showError('请输入有效的端口号（1024-65535）')
      return
    }

    status.value = 'starting'
    toast.dismiss()

    try {
      const result = await startService(url.value.trim(), port.value, allowPortFallback.value)
      serviceUrl.value = result.base_url
      if (result.port_reclaimed) {
        showSuccess(`已释放遗留进程，服务已在端口 ${result.port} 启动`)
      } else if (result.port_changed) {
        port.value = result.port
        showSuccess(`端口 ${result.requested_port} 被占用，已改用 ${result.port}`)
      } else {
        showSuccess('服务已启动')
      }
      status.value = 'running'
      runner.value = 'builtin'
      portHint.value = ''
      persistPrefs()
    } catch (err) {
      toast.showError(String(err))
      status.value = 'error'
    }
  }

  /**
   * 停止正在运行的内置代理服务。
   */
  const handleStop = async (): Promise<void> => {
    status.value = 'stopping'
    toast.dismiss()

    try {
      await stopService()
      status.value = 'idle'
      serviceUrl.value = ''
      runner.value = null
      showSuccess('服务已停止')
    } catch (err) {
      toast.showError(String(err))
      status.value = 'error'
    }
  }

  /**
   * 手动释放被本应用遗留进程占用的端口（无需启动服务）。
   */
  const handleReclaimPort = async (): Promise<void> => {
    if (!port.value) return
    toast.dismiss()
    try {
      await reclaimPort(port.value)
      showSuccess(`端口 ${port.value} 已释放`)
      await refreshPortHint()
    } catch (err) {
      toast.showError(String(err))
    }
  }

  /**
   * 手动触发订阅源刷新，更新本地 clash.yaml 缓存。
   */
  const handleRefresh = async (): Promise<void> => {
    if (!serviceUrl.value) return
    refreshing.value = true
    try {
      const data = await refreshService()
      if (data.ok) {
        showSuccess(
          data.skipped ? '上次刷新仍在进行中' : `已刷新，${data.nodes ?? 0} 个节点`,
        )
      } else {
        toast.showError('刷新失败')
      }
    } catch (err) {
      toast.showError(`刷新失败: ${err}`)
    } finally {
      refreshing.value = false
    }
  }

  /**
   * 将完整订阅地址（base_url + /clash.yaml）复制到系统剪贴板。
   */
  const copyUrl = async (): Promise<void> => {
    if (!serviceUrl.value) return
    try {
      await navigator.clipboard.writeText(`${serviceUrl.value}/clash.yaml`)
      copied.value = true
      showSuccess('已复制订阅地址')
      setTimeout(() => {
        copied.value = false
      }, 2000)
    } catch {
      toast.showError('复制失败')
    }
  }

  /**
   * 将内部状态枚举转为中文展示文案。
   *
   * @param s - 当前 UI 状态
   */
  const formatStatus = (s: ClashUiStatus): string => {
    switch (s) {
      case 'idle': return '空闲'
      case 'starting': return '启动中...'
      case 'running': return '运行中'
      case 'stopping': return '停止中...'
      case 'error': return '错误'
    }
  }

  /**
   * 表单是否应禁用（服务运行中或处于启停过渡态时不允许改 URL/端口）。
   */
  const isFormDisabled =
    (): boolean => status.value === 'running' || status.value === 'starting' || status.value === 'stopping'

  return {
    url,
    port,
    status,
    serviceUrl,
    toast,
    copied,
    refreshing,
    runner,
    portHint,
    portReclaimable,
    allowPortFallback,
    syncStatus,
    handleReclaimPort,
    handleStart,
    handleStop,
    handleRefresh,
    copyUrl,
    formatStatus,
    isFormDisabled,
  }
}

/**
 * 供首页「运行中」徽章使用的轻量查询。
 * 不依赖 composable 实例，避免在 Home 页引入完整状态机。
 *
 * @returns 后端报告 status === 'running' 时为 true；IPC 失败时视为未运行
 */
export const isClashServiceRunning = async (): Promise<boolean> => {
  try {
    const s = await getServiceStatus()
    return s.status === 'running'
  } catch {
    return false
  }
}
