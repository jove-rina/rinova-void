# Round 5 审查 — Clash 体验优化与代码审核

> 审查日期：2026-07-05  
> 对照：Round 4 [10-review-round4-clash.md](./10-review-round4-clash.md) + Batch 4 实现

## Batch 4 落地确认 ✅

| Round 4 项 | 当前状态 | 代码位置 |
|------------|----------|----------|
| EC-14 baseUrl 恢复 | ✅ 部分 | `get_service_status` + `onMounted` |
| EC-17 退出清理 | ✅ | `lib.rs` `RunEvent::Exit` |
| EC-01/02 端口预检 | ✅ | `is_port_listening()` |
| EC-03 stderr 透传 | ✅ | stderr → temp 文件 |
| UX-01 复制地址 | ✅ | `copyUrl()` |
| UX-04 stopping loading | ✅ | template |
| UX-08 手动刷新 | ✅ | `handleRefresh()` fetch POST |
| UX-09 60min 说明 | ✅ | `__interval-note` |
| UX-02 Enter 提交 | ✅ | `@keyup.enter` on URL |
| 无障碍 label/for | ✅ | `sub-url`, `sub-port` |
| `ServiceStatus` 结构体 | ✅ | Rust Serialize + TS interface |

**Clash 工具 UX 评分（Round 5）**

| 维度 | R4 | R5 |
|------|----|----|
| 核心流程 | 4 | **4.5** |
| 错误可理解性 | 2 | **3.5**（stderr 透传） |
| 状态反馈 | 3.5 | **4** |
| 恢复/续用 | 2 | **3**（port/url 仍不全） |
| 操作效率 | 2.5 | **4**（复制+刷新+Enter） |
| 反馈语义 | — | **2.5**（成功信息进 error 区） |
| **综合** | 2.8 | **3.6** |

---

## 一、代码审核

### 1.1 Rust `commands.rs` — 优点

- `kill_child` + `stop_service_impl` 复用，Exit 与 stop 一致 ✅
- `ServiceStatus { status, port, base_url }` 结构化返回 ✅
- `active_port` 独立 Mutex，与 child 生命周期配合 ✅
- 端口预检 + health 双重保障 ✅
- stderr 后台线程读取，避免 pipe 阻塞 ✅

### 1.2 Rust — 待改进

| ID | 问题 | 严重度 | 说明 |
|----|------|--------|------|
| CD-01 | **未持久化 `active_url`** | P1 | 重进页面订阅 URL 仍为空；Rust 只知 port |
| CD-02 | **已在运行错误端口不准** | P2 | L119 用输入 `port`，应读 `active_port` |
| CD-03 | **`get_service_status` 无 health** | P2 | 进程活着但 HTTP 挂掉仍报 running |
| CD-04 | **stderr 读取竞态** | P2 | 进程快退时 thread 可能未写完 temp 文件 |
| CD-05 | **固定 temp 路径** | P3 | `rinova-void-stderr.log` 并发/多实例冲突 |
| CD-06 | **`is_port_listening` 粗粒度** | P3 | 任意 TCP 可连即判占用，无 /health 区分 |

```rust
// CD-02 现状
Ok(None) => return Err(format!("服务已在运行 (port {})", port)),
// 建议
Ok(None) => {
    let active = state.active_port.lock()...;
    return Err(format!("服务已在运行 (port {})", active.unwrap_or(port)));
}
```

### 1.3 Vue `index.vue` — 优点

- 状态机完整，loading 文案含「最多 10 秒」✅
- 复制反馈 `copied` 2s ✅
- running 态禁用表单 ✅
- `role="alert"` + `aria-label` ✅

### 1.4 Vue — 待改进

| ID | 问题 | 严重度 | 说明 |
|----|------|--------|------|
| CD-07 | **刷新成功写入 `errorMsg`** | **P1 UX** | 绿/红语义混乱，成功也显示红框 |
| CD-08 | **onMounted 未恢复 `port`** | P1 | `s.port` 已有但未 `port.value = s.port` |
| CD-09 | **CSP 可能阻断 fetch** | **P1** | `default-src 'self'` 不含 `127.0.0.1` → 手动刷新或失败 |
| CD-10 | starting/stopping 未禁表单 | P2 | 仅 `running` disabled |
| CD-11 | **无 `successMsg` 通道** | P2 | 刷新/复制成功缺正向反馈区 |
| CD-12 | port 无 Enter 提交 | P3 | 仅 URL 字段绑定 enter |
| CD-13 | port NaN 未校验 | P2 | 空端口可能 invoke 0 |
| CD-14 | refresh 用 `fetch` 非 invoke | 设计 | 绕开 Rust，受 CSP 约束；可考虑 Rust proxy |

```typescript
// CD-07 现状 — 成功也进 error 样式
errorMsg.value = data.skipped ? '...' : `已刷新，${data.nodes} 个节点`
```

```typescript
// CD-08 修复建议
if (s.status === 'running' && s.base_url) {
  status.value = 'running'
  serviceUrl.value = s.base_url
  if (s.port) port.value = s.port  // 缺失
}
```

### 1.5 `lib.rs`

```rust
.run(|_app_handle, event| {
    if let RunEvent::Exit = event {
        commands::stop_service_impl(&state);
    }
});
```

✅ 正确模式。注意：仅 `Exit` 事件，窗口 hide 不触发（当前无 tray，可接受）。

### 1.6 Capabilities — 仍开放

```json
["core:default", "core:window:allow-close"]
```

自定义 command ACL 未声明 —  dev 可用，部分 hardened 环境可能拒绝 invoke。

---

## 二、边界情况（Round 5 更新）

| ID | 场景 | R4 | R5 |
|----|------|----|----|
| EC-14 | 重进页面地址正确 | ❌ | ✅ base_url |
| EC-15 | 重进页面 URL 恢复 | ❌ | ❌ 仍缺 active_url |
| EC-15b | 重进页面 port 输入框 | ❌ | ❌ 未赋 `s.port` |
| EC-17 | 退出杀进程 | ❌ | ✅ |
| EC-03 | 启动失败详情 | ❌ | ✅ stderr |
| EC-01 | 端口占用 | ❌ | ✅ 预检 |
| EC-13 | HTTP 死进程活 | ❌ | ❌ |
| EC-20 | 刷新 CSP 拦截 | — | ⚠️ **待验证** CD-09 |

---

## 三、UX 走查（Round 5）

### 3.1 已改善的用户路径 ✅

1. 填 URL → Enter 或点启动 → loading 有预期时间
2. 运行中 → 复制地址 → 「✓ 已复制」
3. 运行中 → 手动刷新 → 有按钮（但反馈颜色有问题）
4. 停止中 → 有 loading 文案
5. 返回首页 → 服务继续（合理），再进入能看到地址

### 3.2 仍影响体验的问题

| 优先级 | 问题 | 建议 |
|--------|------|------|
| P1 | 刷新成功显示红色 error 框 | 增加 `successMsg` + 绿色样式，或 toast |
| P1 | CSP 阻断 localhost fetch | CSP 加 `connect-src 'self' http://127.0.0.1:* http://localhost:*` |
| P1 | 重进后 port 输入框与真实端口不一致 | onMounted 同步 `s.port` |
| P2 | 首页无「Clash 运行中」badge | Home onMounted poll status |
| P2 | 运行中订阅 URL 不可见 | Rust 存 `active_url`，running 态只读展示 |
| P2 | 关闭窗口不提示服务仍运行 | 可选 confirm（低优先） |
| P3 | 刷新失败与启动失败共用 error 区 | 分区或自动清除 |

### 3.3 建议 UI 微调

```
运行中状态下：
┌─────────────────────────────┐
│ 当前订阅（只读）             │  ← 新增，来自 active_url
│ https://xxx...              │
├─────────────────────────────┤
│ 服务地址    [复制地址]       │
│ http://127.0.0.1:25500/...  │
│ [手动刷新]  每60分钟自动刷新  │
└─────────────────────────────┘
成功提示（绿色条，3s 自动消失）  ← 替代 errorMsg 承载成功
```

---

## 四、Batch 5 建议（按优先级）

| # | 任务 | 类型 |
|---|------|------|
| 1 | CSP 添加 `connect-src` localhost | P1 配置 |
| 2 | `successMsg` / 刷新反馈样式分离 | P1 UX |
| 3 | onMounted 恢复 `port`；Rust 存 `active_url` | P1 |
| 4 | 已在运行错误显示 `active_port` | P2 |
| 5 | `get_service_status` 加 health 探活 | P2 |
| 6 | 首页 running badge | P2 |
| 7 | Capabilities command 权限 | P1 安全 |
| 8 | 前端 port NaN 校验 | P2 |

---

## 五、手动回归用例

| # | 步骤 | R5 期望 |
|---|------|---------|
| T1 | 启动 → 复制 → 粘贴 | 完整 URL 含 `/clash.yaml` |
| T2 | 运行中手动刷新成功 | **当前 FAIL**：绿字应在 success 区 |
| T3 | 非 25500 端口启动 → 返回 → 再进 | 地址对，**port 输入框仍 FAIL** |
| T4 | 关闭 Void | 端口释放 ✅ |
| T5 | 占用端口启动 | 明确「端口已被占用」✅ |
| T6 | 错误 URL 启动 | stderr 详情出现在错误框 ✅ |
| T7 | release CSP 下点刷新 | 验证 connect-src（可能 FAIL） |

---

## 相关文档

- [tool-clash-service.md](../tool-clash-service.md)
- [FIX-PLAN.md](./FIX-PLAN.md)
- 上轮：[10-review-round4-clash.md](./10-review-round4-clash.md)
