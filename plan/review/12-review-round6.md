# Round 6 审查 — Clash 工具成熟度评估

> ⚠️ **历史快照**（Batch 6 实施前，2026-07-05）  
> 本文档中的开放项已在 Batch 6 / Phase 3 全部关闭。  
> **当前结案** → [13-review-round7-phase3-closed.md](./13-review-round7-phase3-closed.md)

> 审查日期：2026-07-05  
> 对照：Round 5 [11-review-round5-clash-ux.md](./11-review-round5-clash-ux.md) + Batch 5

## Batch 5 落地确认 ✅

| Round 5 项 | 状态 | 验证 |
|------------|------|------|
| CSP `connect-src` localhost | ✅ | `tauri.conf.json` L27 |
| `successMsg` 绿色反馈 | ✅ | `showSuccess()` + `__success` 样式 |
| Rust `active_url` | ✅ | `ClashServiceState.active_url` |
| onMounted 恢复 port/url | ✅ | `s.port` / `s.url` |
| CD-02 已在运行显示 active_port | ✅ | L125-126 commands.rs |
| port NaN 前端校验 | ✅ | L52-56 index.vue |
| starting/stopping 禁表单 | ✅ | URL/port disabled |

**Clash UX 评分**

| 维度 | R5 | R6 |
|------|----|----|
| 核心流程 | 4.5 | **4.5** |
| 错误/成功反馈 | 2.5 | **4.5** |
| 状态恢复 | 3 | **4.5** |
| 操作效率 | 4 | **4** |
| 全局可感知性 | 2 | **2**（首页仍无 badge） |
| **综合** | 3.6 | **4.1** |

**结论**：Clash 工具已达 **dev/内测可用** 水准；距「可对外 release」差 Capabilities、首页指示、运行态 UI  polish。

---

## 一、代码审核（Round 6）

### 1.1 Rust `commands.rs`（309 行）

**结构清晰，Batch 4–5 建议均已吸收。**

| 模块 | 评价 |
|------|------|
| State 四元组 child/port/url/script | ✅ 完整 |
| stop_service_impl 清三项 | ✅ |
| start_service 流程 | ✅ 校验→预检→spawn→health→持久化 |
| get_service_status | ⚠️ 见 R6-01 |

**R6-01 `get_service_status` 仍不探 health**

进程存活但 HTTP 不可用（SDK 内部 hang）时仍返回 `running`。建议：

```rust
Ok(None) => {
    let alive = port.map(|p| health_check(p).is_ok()).unwrap_or(false);
    if !alive { /* clear child, return stopped */ }
    // else running
}
```

**R6-02 stderr 固定路径**（P3，未变）

`rinova-void-stderr.log` — 多实例/快速重试可能读到旧内容。

**R6-03 成功路径未清 stderr 文件**（P3）

上次失败 stderr 可能在下一次失败时被误读（若进程未写新 stderr）。

### 1.2 Vue `index.vue`（486 行）

**优点**

- 状态机 + 双通道 error/success ✅
- 启动/停止/刷新/复制流程完整 ✅
- 无障碍与动画细节 ✅

**R6-04 运行中 URL 仍可编辑**（P2 UX）

```vue
:disabled="status === 'starting' || status === 'stopping'"
```

`running` 时 URL 输入框**未禁用**，用户可改字但不会影响运行中服务 → 易误解。建议 running 态 URL/port 均 disabled，或改为只读展示块。

**R6-05 复制成功无 success 提示**（P3）

仅 `copied` 按钮文案变化；可考虑 `showSuccess('已复制')` 统一反馈。

**R6-06 组件体积**（P3  refactor）

486 行可抽 `useClashService()` composable（invoke + 状态机），非阻塞。

**R6-07 refresh 仍走 WebView fetch**（设计备注）

CSP 已放行 localhost ✅；若未来收紧 CSP，可改 Rust `refresh_service` command 代理。

### 1.3 全局

| 项 | R6 |
|----|-----|
| Capabilities command ACL | ❌ 仍缺 |
| Home running badge | ❌ |
| README stores/Pinia | ❌ 仍过时 |
| 不规则窗口遮罩 | ❌ |
| 系统 Node 依赖 | ❌ |
| macOS `macos-private-api` 透明窗口警告 | ⚠️ dev 日志可见 |
| `src/api/` invoke 封装 | ❌ |

---

## 二、边界情况（Round 6 更新）

| 场景 | R5 | R6 |
|------|----|----|
| 重进页面 port/url/地址 | ⚠️ | ✅ |
| 刷新成功反馈 | ❌ 红框 | ✅ 绿框 |
| fetch CSP | ⚠️ | ✅ |
| 运行中改 URL 误导 | — | ⚠️ R6-04 |
| HTTP 死进程活 | ❌ | ❌ R6-01 |
| 首页知服务在跑 | ❌ | ❌ |
| 退出清理 | ✅ | ✅ |

---

## 三、UX 走查（Round 6）

### Happy path — 已通过 ✅

填 URL + 端口 → 启动 → 绿字「服务已启动」→ 复制 → 手动刷新 → 绿字节点数 → 停止 → 返回首页再进 → 状态/URL/端口恢复。

### 剩余 UX 缺口

| 优先级 | 问题 | 建议 |
|--------|------|------|
| P1 | 首页无运行指示 | Clash 卡片 badge + 可选绿点 |
| P2 | 运行中 URL 可编辑 | running 禁用或只读展示 |
| P2 | Capabilities 缺失 | permissions 文件 |
| P3 | 复制无 success 条 | showSuccess |
| P3 | 窗口透明 macOS 警告 | `macOSPrivateApi: true` 或文档说明 |

---

## 四、发布就绪度

| 维度 | 评分 | 说明 |
|------|------|------|
| Clash 功能完整度 | **4.5/5** | 主流程 + 恢复 + 反馈齐全 |
| Clash UX | **4/5** | 差首页指示、运行态只读 |
| 后端健壮性 | **4/5** | 差 status health |
| 安全/权限 | **3/5** | Capabilities 未声明 |
| 可分发 release | **3/5** | 需 Node + 无 CI |

**Phase 2（v0.3 Clash 工具）评估：约 90% 完成**

---

## 五、Batch 6 建议

| # | 任务 | 优先级 |
|---|------|--------|
| 1 | Home Clash「运行中」badge（invoke get_service_status） | P1 |
| 2 | running 态 URL/port 只读或 disabled | P2 |
| 3 | `get_service_status` 加 health 二次校验 | P2 |
| 4 | Capabilities 声明三个 command | P1 |
| 5 | README 修正目录树 | P3 |
| 6 | 抽 `useClashService` composable | P3 |

---

## 六、回归用例（Round 6）

| # | 步骤 | 期望 |
|---|------|------|
| T1 | 完整 happy path | ✅ |
| T2 | 返回首页再进 | port/url/地址一致 ✅ |
| T3 | 刷新成功 | 绿色 success 条 ✅ |
| T4 | 运行中改 URL 字段 | **当前可编辑** — 待 R6-04 修复 |
| T5 | 首页 | **无 running 指示** — 待 badge |
| T6 | release 无 Node | 启动失败友好提示 |

---

## 相关文档

- [tool-clash-service.md](../tool-clash-service.md)
- [FIX-PLAN.md](./FIX-PLAN.md)
- 上轮：[11-review-round5-clash-ux.md](./11-review-round5-clash-ux.md)
