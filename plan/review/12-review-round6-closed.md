# Round 6 审查 — 结案（Batch 6 已实施）

> ⚠️ **已被 Round 7  supersede** → [13-review-round7-phase3-closed.md](./13-review-round7-phase3-closed.md)（含 Phase 3 sidecar + 托盘）

> 审查日期：2026-07-05  
> 实施日期：2026-07-05  
> 前置：[11-review-round5-clash-ux.md](./11-review-round5-clash-ux.md)

## 实施摘要

Round 5 / Batch 6 全部待办已在代码中落地。

| 项 | 实现 |
|----|------|
| Capabilities | `permissions/clash-service.toml` + `default.json` |
| 首页 badge | `Home.vue` 5s 轮询 `isClashServiceRunning()` |
| running 表单只读 | `isFormDisabled()` 含 running/starting/stopping |
| status health | `clash.rs` get_service_status 探 health |
| successMsg | 已有 + 复制成功提示 |
| active_url | `ClashServiceState.active_url` |
| refresh | Rust `refresh_service` command |
| SSRF | `validate_subscription_url` 拦截内网 host |
| stderr | 唯一 temp 文件名 + 成功/失败清理 |
| Node | `resolve_node_path()` + README 前置说明 |
| 窗口遮罩 | macOS `EffectsBuilder` radius 24 + `macOSPrivateApi` |
| composable + api | `useClashService.ts` + `api/clash-service.ts` |
| CI | `.github/workflows/ci.yml` |
| backdrop fallback | `main.less` @supports |

## Clash UX 终评

**4.5 / 5** — dev/内测 **可发布**（~~需本机 Node~~ → Phase 3 已内置 sidecar）

## 剩余长期项

- ~~内置 Node sidecar~~ ✅ Phase 3（proxy sidecar）
- ~~系统托盘~~ ✅ Phase 3
- 全局快捷键
- Windows DWM 圆角
- i18n / Vitest

## 新增文件

```
src/api/clash-service.ts
src/composables/useClashService.ts
src-tauri/src/clash.rs
src-tauri/src/window.rs
src-tauri/permissions/clash-service.toml
.github/workflows/ci.yml
```

## 验证

- `pnpm build:proxy` ✅
- `pnpm build` ✅
- `cargo check` ✅
