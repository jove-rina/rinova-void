# 问题清单 — 结案状态

> 2026-07-05 · 前端规范（箭头函数 / Lucide）更新后

## 已全部关闭 ✅

### Batch 6 + Phase 3
- 工具框架、Clash IPC、sidecar、托盘、Capabilities、badge、health、SSRF 等

### Clash UX 优化（Round 14）
- URL/端口 **localStorage** 记忆
- 窗口**拖拽**（无边框标题栏）
- 端口**占用检测**（`check_port`）
- **遗留 proxy 回收**（`reclaim_port` + 启动时自动）
- **优先保留用户端口**（默认不 silent 换端口）
- 可选「占用时自动换端口」

### P0/P1 集成
- **全局快捷键** Cmd/Ctrl+Shift+V
- **窗口位置记忆** tauri-plugin-window-state
- **Vitest** clash-prefs（4 tests）
- **CI** + 签名模板

### 前端规范（2026-07-05）
- **箭头函数**：`src/` 全部函数统一 `const fn = () =>`
- **JSDoc**：模块头 + 类型/函数中文注释
- **Lucide 图标**：`@lucide/vue` 替换 emoji / 内联 SVG
- **registry**：`icon` 类型改为 `LucideIcon` 组件

## 剩余

| 项 | 阶段 |
|----|------|
| 开机自启 | Phase 3/4 |
| 更多工具 | Phase 3+ |
| Windows 原生圆角 | Phase 4 |
| i18n / 主题 / 自动更新 | Phase 4 |
| UI/E2E 测试 | Phase 4 |
| Rust 侧 store 持久化 | 可选 |
| 实际 notarization 流水线跑通 | Phase 4 |

## 成熟度

**Clash 4.5/5** · **桌面集成 4.5/5** · **前端工程 4.5/5**

详见 [02-frontend-review.md](./02-frontend-review.md) · [FIX-PLAN.md](./FIX-PLAN.md)
