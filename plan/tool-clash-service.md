# Clash 订阅服务工具

> 状态：**Rust SDK 内嵌** · [FIX-PLAN.md](./review/FIX-PLAN.md) · [02-frontend-review.md](./review/02-frontend-review.md)

## 功能

本地 HTTP 服务 → `http://127.0.0.1:{port}/clash.yaml`，供 Clash Verge 订阅。[`rinova-proxy-sdk`](https://crates.io/crates/rinova-proxy-sdk) 内嵌于 Tauri 主进程，每 60 分钟自动刷新上游订阅。

**UI**：启动/停止 · 复制 · Rust 侧手动刷新 · 状态恢复 · 首页「运行中」badge · 成功/错误分色 · 运行时标签「内置」· **URL/端口记忆** · **端口占用智能处理** · **Lucide 图标**

## Clash Verge 用法

1. 在 **Void** 中填入机场订阅 URL，点击「启动服务」
2. 复制 Void 显示的地址，形如 `http://127.0.0.1:25500/clash.yaml`
3. 在 **Clash Verge** 中添加远程订阅，粘贴上述本地地址（不要填机场原始链接）

若 Clash 报 `failed to fetch remote profile`，先确认 Void 服务在运行，且浏览器能打开 `/clash.yaml`。

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

## IPC API

| Command | 说明 |
|---------|------|
| `start_service(url, port, allow_fallback?)` | 异步启动内嵌 proxy |
| `stop_service()` | 异步停止服务 |
| `get_service_status()` | `{ status, port, url, base_url, runner }` |
| `refresh_service()` | POST `/refresh`，返回 JSON |
| `check_port(port)` | 检测占用：`available` / `reclaimable` / `foreign` / `suggested_port` |
| `reclaim_port(port)` | 释放疑似**其他** Void 遗留进程占用的端口 |
| `init_window()` | 设置窗口背景色 |

返回 `StartServiceResult`：`{ base_url, port, requested_port, port_changed, port_reclaimed }`

## 端口占用策略

| 情况 | 行为 |
|------|------|
| 端口空闲 | 使用用户指定端口 |
| 占用 + `/health` 正常（疑似**其他** Void 遗留进程） | **自动 kill 并继续用原端口** |
| 占用 + 为本进程当前服务 | 提示「请先在应用内停止服务」，不 kill 自身 |
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
                         rinova-proxy-sdk::start_server (in-process)
                                      ↓
                              /clash.yaml · /health · /refresh
```

- 关闭主窗口 → 隐藏到系统托盘，Clash **继续运行**
- 托盘「退出 Void」→ 停止 Clash 并退出
- 单实例建议：同时开多个 Void 可能导致端口策略混淆

## 安全

- Tauri ACL：`allow-clash-service`
- 订阅 URL SSRF 防护（拒绝 localhost / 私网 IP）
- 本地 HTTP 仅 127.0.0.1
- 端口释放不会 kill 当前进程 PID

## 构建与依赖

```bash
pnpm tauri:dev      # 开发
pnpm tauri:build    # release，无需 Node 运行时
```

Rust 依赖：`rinova-proxy-sdk = "1.0.0"`（见 `src-tauri/Cargo.toml`）

## 文件

| 路径 | 职责 |
|------|------|
| `src/tools/clash-service/index.vue` | UI 壳 + Lucide 图标 + 端口提示/释放/换端口 |
| `src/composables/useClashService.ts` | 状态机 + prefs + 端口检测 |
| `src/utils/clash-prefs.ts` | URL/端口 localStorage |
| `src/api/clash-service.ts` | typed invoke 封装 |
| `src/tools/registry.ts` | 工具元数据 + `LucideIcon` 注册 |
| `src/components/WindowHeader.vue` | 无边框窗口拖拽区 + 关闭 |
| `src-tauri/src/clash.rs` | 内嵌服务 / HTTP / SSRF / 端口回收 |
| `src-tauri/src/window.rs` | 窗口背景色 |
| `src-tauri/src/tray.rs` | 系统托盘 |
| `src-tauri/permissions/clash-service.toml` | ACL |
| `public/favicon.svg` | Lucide CircleDot 品牌 favicon |
