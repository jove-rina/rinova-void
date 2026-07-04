# Rust / Tauri 后端审查

> Round 7 · 后续 UX → [14-review-clash-ux-polish.md](./14-review-clash-ux-polish.md)

## 模块结构

| 模块 | 行数 | 职责 | 状态 |
|------|------|------|------|
| `commands.rs` | 35 | 薄 command 层 | ✅ |
| `clash.rs` | 660 | 进程/HTTP/SSRF/端口回收 | ✅ |
| `sidecar.rs` | 110 | sidecar 解析 + spawn | ✅ |
| `tray.rs` | 75 | 系统托盘 | ✅ |
| `window.rs` | 32 | macOS 圆角 | ✅ |
| `lib.rs` | 60 | setup + 事件循环 | ✅ |

## clash.rs

| 能力 | 状态 |
|------|------|
| active_url/port/child 持久化 | ✅ |
| 端口预检 + health 启动 | ✅ |
| stderr 诊断（唯一 temp） | ✅ |
| get_service_status + health | ✅ |
| SSRF validate_subscription_url | ✅ |
| refresh_service Rust HTTP | ✅ |
| ProxyRunner sidecar / Node | ✅ |
| check_port / reclaim_port | ✅ |
| prepare_listen_port（回收 + 可选 fallback） | ✅ |

## lib.rs 生命周期

| 事件 | 行为 | 状态 |
|------|------|------|
| CloseRequested | hide + prevent_close（Clash 继续） | ✅ Phase 3 |
| ExitRequested (code None) | prevent_exit（Windows 保活） | ✅ |
| Exit / tray-quit | stop_service_impl | ✅ |

## sidecar.rs

| 能力 | 状态 |
|------|------|
| resolve_sidecar_path（exe / dev / resource） | ✅ |
| resolve_runner 回退 Node | ✅ |
| spawn_proxy 统一入口 | ✅ |

## 评分

| 维度 | R6 | R7 |
|------|----|----|
| 状态 API | 4.5 | **4.5** |
| 进程管理 | 4.5 | **4.5** |
| 模块结构 | 3.5 | **5** |
| 桌面集成 | 2 | **4** |
| 发布就绪 | 3.5 | **4.5** |

## 剩余

- 全局快捷键 plugin
- 全平台 sidecar 构建矩阵（CI macOS/Windows）
