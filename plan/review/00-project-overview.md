# 项目概览

> 最后更新：2026-07-05 · 前端规范（箭头函数 / JSDoc / Lucide）

## 产品定位

**Void（虚空口袋）** — Tauri 2 桌面小工具集合。首个工具 **Clash 订阅服务** 完整可用；release 内置 proxy sidecar。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | Tauri 2.11 + 系统托盘 + 全局快捷键 |
| 前端 | Vue 3 + composables + Vue Router + **@lucide/vue** |
| 样式 | Less + CSS 变量（`main.less`） |
| 语言 | TypeScript strict；前端函数统一箭头函数 + JSDoc |
| 后端 | Rust（clash / sidecar / tray / window / shortcut） |
| 子进程 | pkg sidecar 或 Node 回退 |

## 目录结构（节选）

```
src/
├── api/clash-service.ts        # typed invoke（箭头函数）
├── composables/useClashService.ts
├── utils/clash-prefs.ts        # URL/端口记忆
├── tools/registry.ts           # 工具注册 + LucideIcon
├── views/Home.vue
└── components/WindowHeader.vue
public/favicon.svg              # Lucide CircleDot
src-tauri/src/clash.rs          # 含端口回收
src-tauri/src/tray.rs
```

## 运行状态

| 检查 | 结果 |
|------|------|
| build / sidecar / cargo check | ✅ |
| Clash prefs + 端口回收 | ✅ |
| 系统托盘 + sidecar | ✅ |
| 全局快捷键 + 窗口位置记忆 | ✅ |
| Lucide 图标全量替换 | ✅ |
| Vitest（clash-prefs） | ✅ 4 tests |

## 前端规范（2026-07-05）

- 所有函数使用箭头函数（含 `export const useXxx = () =>`）
- 模块与公共 API 配备中文 JSDoc
- UI 图标统一 `@lucide/vue`，registry 存 `LucideIcon` 组件

## 已知限制

- 端口回收依赖 `/health` 识别（非 Void 服务不会自动 kill）
- Windows 原生圆角未实现
- 开机自启、自动更新、i18n 待 Phase 4

详见 [02-frontend-review.md](./02-frontend-review.md) · [FIX-PLAN.md](./FIX-PLAN.md)
