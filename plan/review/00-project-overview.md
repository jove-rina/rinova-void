# 项目概览

> 最后更新：2026-07-05 · Clash UX 优化

## 产品定位

**Void（虚空口袋）** — Tauri 2 桌面小工具集合。首个工具 **Clash 订阅服务** 完整可用；release 内置 proxy sidecar。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | Tauri 2.11 + 系统托盘 |
| 前端 | Vue 3 + composables + localStorage prefs |
| 后端 | Rust（clash / sidecar / tray / window） |
| 子进程 | pkg sidecar 或 Node 回退 |

## 目录结构（节选）

```
src/utils/clash-prefs.ts      # URL/端口记忆
src/composables/useClashService.ts
src-tauri/src/clash.rs        # 含端口回收
src-tauri/src/tray.rs
```

## 运行状态

| 检查 | 结果 |
|------|------|
| build / sidecar / cargo check | ✅ |
| Clash prefs + 端口回收 | ✅ |
| 系统托盘 + sidecar | ✅ |

## 已知限制

- 全局快捷键未实现
- 端口回收依赖 `/health` 识别（非 Void 服务不会自动 kill）

详见 [14-review-clash-ux-polish.md](./review/14-review-clash-ux-polish.md)
