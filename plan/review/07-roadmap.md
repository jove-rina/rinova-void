# 功能路线图

基于代码现状、注释意图和产品定位，建议的开发里程碑。

> 最后更新：2026-07-05 · Round 7

---

## Phase 0：基础完善（v0.1.0）— ✅ 完成

| 任务 | 状态 | 说明 |
|------|------|------|
| 无边框透明窗口 | ✅ | tauri.conf.json |
| 自定义标题栏 | ✅ | 拖拽 + 关闭 |
| 毛玻璃 UI | ✅ | CSS backdrop-filter + 降级 |
| 工具列表 UI | ✅ | registry 驱动 |
| 版本号 / README / Vite / CSP | ✅ | Round 2–6 |
| 工具注册表 + Router | ✅ | Round 3 |
| IPC commands | ✅ | Round 3 → clash 模块 |
| 首个工具 Clash 服务 | ✅ | 完整可用 |
| 生产运行时链 | ✅ | Phase 3 sidecar |
| Command Capabilities | ✅ | Batch 6 |
| macOS 窗口圆角 | ✅ | Batch 6 |

**交付标准**：✅ 可独立 release 包（内置 proxy sidecar）

---

## Phase 1：工具框架（v0.2.0）— ✅ 完成

| 任务 | 状态 |
|------|------|
| 工具注册表 + registry 路由 | ✅ |
| IPC + 返回导航 | ✅ |
| typed api + composable | ✅ |
| URL/端口 localStorage 记忆 | ✅ |
| 配置持久化 Rust store | ⬜ 长期（localStorage 已够用） |

---

## Phase 2：首个真实工具（v0.3.0）— ✅ 完成

> Clash UX **4.5/5**

| 任务 | 状态 |
|------|------|
| Clash 全流程 + esbuild + health | ✅ |
| 状态恢复 + prefs + 端口回收 | ✅ |
| Capabilities / SSRF / refresh Rust | ✅ |
| macOS 窗口圆角 | ✅ |
| 内置 proxy sidecar | ✅ Phase 3 |

---

## Phase 3：系统集成（v0.4.0）— 进行中（~75%）

| 任务 | 状态 |
|------|------|
| 内置 proxy sidecar（pkg） | ✅ |
| 系统托盘（关闭隐藏、菜单退出） | ✅ |
| 全局快捷键 | ✅ Cmd/Ctrl+Shift+V |
| 窗口位置记忆 | ✅ window-state |
| 开机自启 | ⬜ |
| 更多工具 | ⬜ |

**交付标准**：关闭窗口不退出 ✅ · 快捷键唤起 ✅

---

## Phase 4：polish & 发布（v1.0.0）

| 任务 | 状态 | 说明 |
|------|------|------|
| CI/CD | ✅ | test + ubuntu/macos matrix |
| Vitest + Rust tests | ✅ | prefs + clash SSRF/scan |
| 代码签名 | ⬜ 模板就绪 | secrets + workflow_dispatch |
| 自动更新 | ⬜ | tauri-plugin-updater |
| 主题切换 | ⬜ | 亮/暗主题 |
| 无障碍 | ⬜ 部分 | 部分 ARIA |
| 性能优化 | ⬜ | 启动速度、sidecar 体积 |
| 文档 | ✅ 部分 | plan/review + README |
| CSP 配置 | ✅ 基础 | connect-src localhost |

**交付标准**：GitHub Releases 可下载安装

---

## Phase 5：生态扩展（v1.x）

| 方向 | 说明 |
|------|------|
| 工具插件 API | 第三方工具开发规范 |
| 工具市场 | 在线工具列表（可选） |
| 云同步 | 配置跨设备同步（可选） |
| 移动端 | iOS / Android 适配 |
| 国际化 | 多语言支持 |

---

## 技术决策记录

### 已确定

| 决策 | 选择 | 理由 |
|------|------|------|
| 桌面框架 | Tauri 2 | 轻量、Rust 安全、跨平台 |
| 前端框架 | Vue 3 | 简洁、Composition API |
| 包管理 | pnpm | 快速、节省磁盘 |
| 样式方案 | Less + CSS 变量 | 简单够用 |
| 窗口风格 | 透明无边框 | 产品差异化 |
| 路由 | Vue Router | registry 驱动 |
| 状态管理 | composables | 当前规模足够 |
| Clash 运行时 | pkg sidecar | 免用户 Node |
| 托盘行为 | 关闭 = 隐藏到托盘 | Phase 3 已实施 |

### 待决定

| 决策 | 选项 | 建议 |
|------|------|------|
| 全局快捷键 | tauri-plugin-global-shortcut | Cmd+Shift+V |
| License | MIT vs Apache-2.0 | MIT |
| 配置持久化 | Pinia + tauri-plugin-store | 工具增多后引入 |

---

## 时间估算（参考）

| Phase | 状态 | 预估 |
|-------|------|------|
| Phase 0–2 | ✅ | — |
| Phase 3 剩余 | 进行中 | 1–2 天（快捷键 + 窗口记忆） |
| Phase 4 | 未开始 | 5–7 天 |
