# 问题清单 — 结案状态

> 2026-07-05 · Clash UX 优化后

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

## 剩余

| 项 | 阶段 |
|----|------|
| 全局快捷键 | Phase 3 |
| 窗口位置记忆 | Phase 3 |
| Rust 侧 store 持久化 | 可选 |
| i18n / 签名 / Vitest | Phase 4 |

## 成熟度

**Clash 4.5/5** · **桌面集成 4/5**

详见 [14-review-clash-ux-polish.md](./14-review-clash-ux-polish.md)
