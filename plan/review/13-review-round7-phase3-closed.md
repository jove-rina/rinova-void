# Round 7 审查 — Phase 3 结案

> 审查日期：2026-07-05  
> 对照：Batch 6 [12-review-round6-closed.md](./12-review-round6-closed.md) + Phase 3 实施

## 审查结论

| 阶段 | 状态 | 评分 |
|------|------|------|
| Phase 0 基础完善 | ✅ 完成 | — |
| Phase 1 工具框架 | ✅ 完成 | ~95% |
| Phase 2 Clash 工具 | ✅ 完成 | UX **4.5/5** |
| Phase 3 系统集成 | **部分完成** | sidecar + 托盘 ✅ |

**可分发 release**：`pnpm tauri:build` 含内置 proxy sidecar，**无需用户安装 Node**。

---

## Batch 6 回归确认 ✅

| Round 6 待办 | 代码验证 | 状态 |
|--------------|----------|------|
| Capabilities `allow-clash-service` | `permissions/clash-service.toml` + `default.json` | ✅ |
| 首页 running badge | `Home.vue` 5s 轮询 | ✅ |
| running/starting/stopping 表单禁用 | `useClashService.isFormDisabled()` | ✅ |
| `get_service_status` health 二次校验 | `clash.rs` health_check | ✅ |
| `useClashService` + `api/clash-service.ts` | 已拆分 | ✅ |
| `refresh_service` Rust command | `clash.rs` http_post_json | ✅ |
| SSRF 内网拦截 | `validate_subscription_url` | ✅ |
| stderr 唯一 temp + 清理 | `stderr_temp_path()` | ✅ |
| Node 路径探测 | `resolve_node_path()`（dev 回退） | ✅ |
| macOS 窗口圆角 | `window.rs` + `macOSPrivateApi` | ✅ |
| backdrop-filter 降级 | `main.less` @supports | ✅ |
| GitHub Actions CI | `ci.yml` build + sidecar + cargo check | ✅ |

**Round 6 文档中的开放项（R6-01~R6-07）均已关闭。**

---

## Phase 3 新增确认 ✅

| 任务 | 实现 | 状态 |
|------|------|------|
| 内置 proxy sidecar | `scripts/build-sidecar.mjs` + `@yao-pkg/pkg` | ✅ |
| externalBin 打包 | `tauri.conf.json` `binaries/proxy-server` | ✅ |
| sidecar 路径解析 | `src-tauri/src/sidecar.rs` | ✅ |
| Node 回退 | sidecar 不存在时用系统 Node | ✅ |
| runner 标签 | `ServiceStatus.runner` → UI「内置/Node」 | ✅ |
| 系统托盘 | `src-tauri/src/tray.rs` TrayIconBuilder | ✅ |
| 关闭隐藏到托盘 | `on_window_event` prevent_close + hide | ✅ |
| 托盘退出清理 | `tray-quit` → stop_service + exit(0) | ✅ |
| Windows 保活 | `ExitRequested` prevent_exit (code None) | ✅ |

### 行为变更（相对 Batch 6 文档）

| 事件 | Batch 6 | Phase 3（当前） |
|------|---------|-----------------|
| 窗口 × 关闭 | 停止 Clash + 退出 | **隐藏到托盘**，Clash **继续运行** |
| 完全退出 | RunEvent::Exit | 托盘「退出 Void」或 `app.exit(0)` |

---

## 代码审核（Round 7）

### Rust 模块结构

```
lib.rs       → 托盘 setup、关窗隐藏、Exit 清理
clash.rs     → 进程/HTTP/SSRF/status（446 行）
sidecar.rs   → sidecar 解析 + spawn
tray.rs      → 菜单 + 左键切换
window.rs    → macOS 圆角
commands.rs  → 薄 command 层（35 行）
```

| 维度 | 评分 | 说明 |
|------|------|------|
| 模块划分 | **5/5** | clash/sidecar/tray 职责清晰 |
| 进程管理 | **4.5/5** | SIGTERM + health + sidecar 双路径 |
| 桌面集成 | **4/5** | 托盘完整；缺全局快捷键 |
| 发布就绪 | **4.5/5** | sidecar + CI；未验 `tauri:build` 全平台 |

### 前端

| 项 | 状态 |
|----|------|
| composable 状态机 | ✅ |
| runner 标签（内置/Node） | ✅ |
| 首页 badge | ✅ |
| success/error 分通道 | ✅ |

---

## 边界情况（Round 7）

| 场景 | 状态 |
|------|------|
| release 无系统 Node | ✅ sidecar 启动 |
| dev 无 sidecar | ✅ 回退 Node + 友好错误 |
| 关窗后 Clash 仍可用 | ✅ |
| 托盘退出停止 Clash | ✅ |
| 重进页面状态恢复 | ✅ |
| HTTP 死进程活 | ✅ health 二次校验 |
| 运行中改 URL 误导 | ✅ 表单 disabled |
| 首页 running 指示 | ✅ badge |

---

## 仍开放（Phase 3+）

| 优先级 | 项 | 阶段 |
|--------|-----|------|
| P2 | 全局快捷键 Cmd+Shift+V | Phase 3 |
| P3 | 窗口位置记忆 | Phase 3 |
| P3 | 开机自启 | Phase 3 |
| P3 | Windows DWM 原生圆角 | 长期 |
| P4 | 代码签名 / 自动更新 / i18n | Phase 4 |
| P4 | Vitest 单元测试 | Phase 4 |

---

## 验证

| 命令 | 结果 |
|------|------|
| `pnpm build:proxy` | ✅ |
| `pnpm build:sidecar` | ✅（本机 x86_64-apple-darwin） |
| `pnpm build` | ✅ |
| `cargo check` | ✅ |
| `pnpm tauri:build` | ⚠️ 未在本轮全平台验证 |

---

## 相关文档

- [FIX-PLAN.md](./FIX-PLAN.md)
- [07-roadmap.md](./07-roadmap.md)
- [14-review-clash-ux-polish.md](./14-review-clash-ux-polish.md)（prefs / 端口 / 拖拽）
- [tool-clash-service.md](../tool-clash-service.md)
