# Void — 审核修复计划

> 最近更新：2026-07-05 · **前端规范 + Lucide** 完成

## 已完成 ✅

### 工程基础（Batch 1–3）
- [x] CSP、Vite、registry、commands、esbuild、health

### Clash Batch 4–5
- [x] ServiceStatus、Exit 清理、端口预检、复制/刷新、successMsg、active_url、CSP connect-src

### Batch 6（Round 6 全部待办）
- [x] Capabilities、首页 badge、表单禁用、health status、composable、SSRF、refresh Rust、窗口圆角、CI 等

### Phase 3（2026-07-05）
- [x] 内置 proxy sidecar（pkg + externalBin）
- [x] 系统托盘（关窗隐藏、托盘退出清理）
- [x] runner 标签、Windows ExitRequested 保活

### Clash UX 优化（2026-07-05）
- [x] **URL/端口 localStorage 记忆**（`clash-prefs.ts`）
- [x] **窗口拖拽**（`core:window:allow-start-dragging` + startDragging）
- [x] **`check_port`** — 区分 reclaimable / foreign
- [x] **`reclaim_port`** — 释放 Void 遗留 proxy
- [x] **优先保留用户端口** — 遗留进程自动回收；默认不 silent fallback
- [x] **可选「占用时自动换端口」** — `allow_fallback` 参数

### 前端规范（2026-07-05）
- [x] **箭头函数** — `src/` 全部函数统一箭头函数风格
- [x] **JSDoc 注释** — 模块头 + 类型/函数中文文档
- [x] **Lucide 图标** — `@lucide/vue` 全量替换 emoji / 内联 SVG
- [x] **registry 图标类型** — `icon: LucideIcon` 组件注册
- [x] **favicon** — `public/favicon.svg` 使用 CircleDot

### P0/P1（2026-07-05）
- [x] **全局快捷键** Cmd/Ctrl+Shift+V（`shortcut.rs`）
- [x] **窗口位置记忆** `tauri-plugin-window-state`
- [x] **Rust 单元测试** 8 tests（SSRF + 端口扫描）
- [x] **Vitest** 4 tests（clash-prefs）
- [x] **CI** ubuntu + macOS matrix；`release-build` workflow 模板
- [x] **签名配置** `tauri.conf.json` + README secrets 说明

> **行为**：窗口 × → 隐藏托盘（Clash 继续）；托盘退出 → 停止 Clash。

---

## 剩余

- [ ] 开机自启（可选）
- [ ] 更多工具
- [ ] Windows 原生圆角
- [ ] 国际化、主题、自动更新（Phase 4）
- [ ] 实际 notarization / 证书流水线跑通
- [ ] Rust 侧 prefs（tauri-plugin-store，可选）

---

## 批次记录

| 批次 | 日期 | 内容 |
|------|------|------|
| Batch 1–5 | 07-04~05 | 工程 + Clash 核心 UX |
| Batch 6 | 07-05 | Round 6 审查待办 |
| Phase 3 | 07-05 | sidecar + 托盘 |
| **Clash UX** | 07-05 | prefs + 端口回收 + 拖拽 |
| **Frontend** | 07-05 | 箭头函数 + JSDoc + Lucide |
| **P0/P1** | 07-05 | 快捷键 + window-state + tests + CI |
| Review 7 | 07-05 | [13-review-round7-phase3-closed.md](./13-review-round7-phase3-closed.md) |
| Review 14 | 07-05 | [14-review-clash-ux-polish.md](./14-review-clash-ux-polish.md) |
