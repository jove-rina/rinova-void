# Clash UX 优化 — 审查记录

> 日期：2026-07-05 · 接续 [13-review-round7-phase3-closed.md](./13-review-round7-phase3-closed.md)

## 实施摘要

| 项 | 实现 | 状态 |
|----|------|------|
| URL/端口记忆 | `clash-prefs.ts` localStorage | ✅ |
| 窗口拖拽 | `allow-start-dragging` + `WindowHeader` startDragging | ✅ |
| 端口占用检测 | `check_port` command | ✅ |
| 遗留进程回收 | `reclaim_port` + 启动时 `prepare_listen_port` | ✅ |
| 优先保留原端口 | 默认不 fallback；仅 `allow_fallback=true` 时换端口 | ✅ |
| UI 端口提示 | 占用类型分色提示 + 释放按钮 + 换端口勾选 | ✅ |

---

## 端口策略（最终）

```
用户指定 port
    ↓
空闲? ──是──→ 使用 port
    ↓ 否
/health 正常? ──是──→ kill 占用进程 → 使用 port（port_reclaimed）
    ↓ 否
allow_fallback? ──是──→ 扫描 port+1 … +32
    ↓ 否
报错，提示关闭占用程序
```

**设计理由**：Clash 常固定配置 `25500`；Void 异常退出可能导致遗留 proxy 占端口，应回收而非静默换端口。

---

## IPC 变更

| Command | 新增/变更 |
|---------|-----------|
| `start_service` | 增参 `allow_fallback?: bool`；返回 `port_reclaimed` |
| `check_port` | 返回 `reclaimable` / `foreign` |
| `reclaim_port` | 新增 |

---

## 边界情况

| 场景 | 行为 |
|------|------|
| 重启 app，URL 仍记得 | ✅ localStorage |
| 25500 被 Void 遗留占用 | ✅ 启动时自动释放并用 25500 |
| 25500 被其他程序占用 | ✅ 报错；可选换端口 |
| 关窗到托盘后再开 | ✅ 服务仍 running，状态同步 |
| 无边框窗口拖拽 | ✅ 标题栏可拖 |

---

## 相关文档

- [tool-clash-service.md](../tool-clash-service.md)
- [FIX-PLAN.md](./FIX-PLAN.md)
- [02-frontend-review.md](./02-frontend-review.md) — 后续前端规范（箭头函数 / JSDoc / Lucide）

---

## 附录：后续前端更新（同日）

| 项 | 说明 |
|----|------|
| Lucide 图标 | `@lucide/vue` 替换 emoji / 内联 SVG；registry `icon: LucideIcon` |
| 箭头函数 | `src/` 全部函数统一箭头函数 |
| JSDoc | 模块与 API 中文注释补全 |

本表为 Round 14 之后的增量，不改变上文 Clash UX 端口/拖拽结论。
