# Clash 订阅服务工具

> 状态：**Phase 3 + UX 优化 + 前端规范** · [FIX-PLAN.md](./review/FIX-PLAN.md) · [02-frontend-review.md](./review/02-frontend-review.md)

## 功能

本地 HTTP 服务 → `http://127.0.0.1:{port}/clash.yaml`，供 Clash Verge 订阅。SDK 每 60 分钟自动刷新。

**UI**：启动/停止 · 复制 · Rust 侧手动刷新 · 状态恢复 · 首页「运行中」badge · 成功/错误分色 · 运行时标签（内置/Node）· **URL/端口记忆** · **端口占用智能处理** · **Lucide 图标**

## UI 图标（Lucide）

| 位置 | 组件 | 文件 |
|------|------|------|
| 首页品牌 | `CircleDot` | `Home.vue` |
| 工具卡片 | `Shield`（registry） | `registry.ts` / `Home.vue` |
| 卡片箭头 | `ChevronRight` | `Home.vue` |
| 标题栏关闭 | `X` | `WindowHeader.vue` |
| 返回 | `ChevronLeft` | `clash-service/index.vue` |
| 启动 / 停止 | `Play` / `Square` | `clash-service/index.vue` |
| 加载中 | `Loader2`（旋转） | `clash-service/index.vue` |
| 复制 / 已复制 | `Copy` / `Check` | `clash-service/index.vue` |
| 手动刷新 | `RefreshCw`（刷新时旋转） | `clash-service/index.vue` |
| favicon | `CircleDot` SVG | `public/favicon.svg` |

新增工具时在 `registry.ts` 的 `icon` 字段填入 Lucide 组件（类型 `LucideIcon`）。

## IPC API

| Command | 说明 |
|---------|------|
| `start_service(url, port, allow_fallback?)` | 启动 proxy；默认 `allow_fallback=false` |
| `stop_service()` | 停止服务 |
| `get_service_status()` | `{ status, port, url, base_url, runner }` + health |
| `refresh_service()` | POST `/refresh`，返回 JSON |
| `check_port(port)` | 检测占用：`available` / `reclaimable` / `foreign` / `suggested_port` |
| `reclaim_port(port)` | 释放疑似 Void 遗留进程占用的端口 |
| `init_window()` | macOS 窗口圆角效果 |

返回 `StartServiceResult`：`{ base_url, port, requested_port, port_changed, port_reclaimed }`

前端封装：`src/api/clash-service.ts`（箭头函数 + JSDoc）· composable：`src/composables/useClashService.ts`

## 端口占用策略

| 情况 | 行为 |
|------|------|
| 端口空闲 | 使用用户指定端口 |
| 占用 + `/health` 正常（疑似 Void 遗留） | **自动 kill 并继续用原端口** |
| 被其他程序占用 | 默认报错；勾选「占用时自动换端口」后向上扫描最多 32 个 |
| 手动 | 「立即释放此端口」按钮（`reclaim_port`） |

识别遗留服务：对占用端口做 `GET /health`，仅当响应符合 proxy 预期时才自动释放。

## 偏好持久化

- `src/utils/clash-prefs.ts` — `localStorage` 键 `void.clash.prefs`
- 保存/恢复：**订阅 URL**、**端口**
- 应用重启后自动填充；服务运行中仍从 Rust `active_url` 同步

## 架构

```
index.vue → useClashService → api/clash-service → commands.rs → clash.rs
                                      ↓
                         sidecar.rs → proxy-server (pkg) 或 node + bundle.cjs
                                      ↓
                              @rinova/proxy-sdk → /clash.yaml
```

- 关闭主窗口 → 隐藏到系统托盘，Clash **继续运行**
- 托盘「退出 Void」→ 停止 Clash 并退出
- 标题栏拖拽：`core:window:allow-start-dragging` + `startDragging()`

## 安全

- Tauri ACL：`allow-clash-service`
- 订阅 URL SSRF 防护（拒绝 localhost / 私网 IP）
- 本地 HTTP 仅 127.0.0.1
- 端口释放仅针对 health 匹配的本地 proxy，不盲目 kill

## 构建与依赖

```bash
pnpm build:proxy    # esbuild bundle（含 SDK）
pnpm build:sidecar  # pkg → src-tauri/binaries/proxy-server-{triple}
pnpm tauri:build    # release 含 sidecar，无需用户安装 Node
pnpm tauri:dev      # 开发可回退系统 Node
```

前端图标依赖：`@lucide/vue`（见 `package.json`）。

## 文件

| 路径 | 职责 |
|------|------|
| `src/tools/clash-service/index.vue` | UI 壳 + Lucide 图标 + 端口提示/释放/换端口 |
| `src/composables/useClashService.ts` | 状态机 + prefs + 端口检测（箭头函数） |
| `src/utils/clash-prefs.ts` | URL/端口 localStorage |
| `src/api/clash-service.ts` | typed invoke 封装 |
| `src/tools/registry.ts` | 工具元数据 + `LucideIcon` 注册 |
| `src/components/WindowHeader.vue` | 无边框窗口拖拽区 + 关闭 |
| `src-tauri/src/clash.rs` | 进程/HTTP/SSRF/端口回收 |
| `src-tauri/src/sidecar.rs` | sidecar 路径与 spawn |
| `src-tauri/src/tray.rs` | 系统托盘 |
| `src-tauri/src/window.rs` | macOS 圆角 |
| `scripts/build-sidecar.mjs` | pkg 打包脚本 |
| `src-tauri/permissions/clash-service.toml` | ACL |
| `public/favicon.svg` | Lucide CircleDot 品牌 favicon |
