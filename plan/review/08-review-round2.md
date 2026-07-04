# 第二轮审查报告

> 审查日期：2026-07-04（Round 2）  
> 文档路径：`plan/review/`  
> 对照基准：Round 1 审查 + [FIX-PLAN.md](./FIX-PLAN.md) 已执行项

## 审查范围

本轮在文档迁移至 `plan/review/` 后，对**当前代码库**重新走查，验证 FIX-PLAN 落地情况，并更新各专项文档中的过时描述。

## 与 Round 1 的变化摘要

| 类别 | Round 1 | Round 2（当前） |
|------|---------|-----------------|
| CSP | `null` | ✅ 已配置明确策略 |
| Cargo.toml 元信息 | 模板占位 | ✅ 已填写（rinova-void / MIT / Rina） |
| vite.config.ts | 仅 `plugins: [vue()]` | ✅ Tauri 推荐项 + `@` 别名 |
| tsconfig paths | 无 | ✅ `@/*` 映射 |
| WindowHeader 无障碍 | 无 aria-label | ✅ 已添加 |
| 关闭按钮拖拽冲突 | 未处理 | ✅ `-webkit-app-region: no-drag` |
| App.vue 初始化 | 空 `onMounted` | ⚠️ 改为 `ready` ref，但仍无实际逻辑 |
| package.json 版本 | 0.0.0 | ❌ 仍为 0.0.0 |
| README | Vue 模板 | ❌ 仍为模板 |
| 不规则窗口遮罩 | 未实现 | ❌ 仍未实现 |
| 工具系统 | 占位 | ❌ 仍未实现 |
| IPC 层 | 无 | ❌ 仍无 |

## 新发现问题

### R2-1：App.vue `ready` 为无效状态（P2）

```typescript
const ready = ref(false)

Promise.resolve().then(() => {
  ready.value = true
})
```

- `ready` 未在 template 或业务逻辑中读取
- 仅为占位，等价于 Round 1 的空 `onMounted`，且引入多余响应式开销
- **建议**：删除 `ready` 与 `Promise.resolve()`；待 Rust 遮罩 command 就绪后再加真实初始化

### R2-2：FIX-PLAN 条目重复（文档）

- P1 已勾选「vite.config.ts 加 `@` 路径别名」
- P2 又列出同一项为未完成 — 文档自相矛盾
- **建议**：合并为一条，P2 仅保留 App.vue scoped 等待办

### R2-3：三处版本源仍未对齐（P1）

| 文件 | version |
|------|---------|
| `package.json` | **0.0.0** |
| `tauri.conf.json` | 0.1.0 |
| `Cargo.toml` | 0.1.0 |

Tauri 打包以 `tauri.conf.json` 为准，但 npm 生态与 CI 通常读 `package.json`，不一致易造成发布混乱。

### R2-4：`@` 别名已配置但未使用（P3）

- `vite.config.ts` 与 `tsconfig.app.json` 均已配置 `@`
- 所有 import 仍为相对路径（如 `./components/WindowHeader.vue`）
- 无功能问题，但配置处于「预置未启用」状态

### R2-5：审查文档与代码漂移（流程）

Round 1 文档在 FIX-PLAN 执行后未同步更新，导致：
- `02-frontend-review.md` 仍写 Vite 缺失、Header 无 aria-label
- `03-backend-review.md` 仍写 Cargo 模板名、`csp: null`
- `04-security-review.md` CSP 评分仍为 2

**建议**：每次 FIX-PLAN 批次完成后更新对应专项文档，或在本文件记录 delta。

## 已验证项

| 检查 | 结果 |
|------|------|
| `pnpm build` | ✅ 2026-07-04 通过 |
| vue-tsc 严格模式 | ✅ 通过（含 noUnusedLocals） |
| CSP 字符串合法性 | ✅ 与 Round 1 建议一致 |
| Capabilities 最小权限 | ✅ 未变，仍合理 |
| 构建产物体积 | ~77 KB JS（gzip ~28 KB） |

## 更新后的评分对比

| 维度 | Round 1 | Round 2 | 变化原因 |
|------|---------|---------|----------|
| 配置完整度 | 2 | 4 | CSP、Vite、Cargo 元信息补齐 |
| 前端代码质量 | 4 | 3.5 | `ready` 死代码拉低 |
| 安全（CSP） | 2 | 4 | CSP 已配置 |
| 产品可用性 | 1 | 1 | P0 功能仍未实现 |
| 文档准确度 | — | 3 | 审查文档曾滞后于代码 |

## 当前开放问题统计

| 优先级 | Round 1 | Round 2 | 说明 |
|--------|---------|---------|------|
| P0 | 2 | **2** | 窗口遮罩、工具系统 |
| P1 | 5 | **3** | 剩版本号、README、IPC |
| P2 | 8 | **6** | 修复 2 项；新增 ready 死代码 |
| P3 | 7 | **7** | 无变化 |
| **开放合计** | 22 | **18** | 已关闭 4 项 |

## 建议下一步（按顺序）

1. **P1** `package.json` version → `0.1.0`
2. **P2** 删除 App.vue 中无效的 `ready` ref
3. **P0** 实现 Rust 窗口圆角遮罩 + 前端 invoke 调用
4. **P0** 建立 `src/tools/registry.ts` 与首个工具
5. **P1** 重写 README
6. **P2** App.vue 样式改 scoped（或改模块 CSS）

详见更新后的 [FIX-PLAN.md](./FIX-PLAN.md) 与 [06-issues-and-recommendations.md](./06-issues-and-recommendations.md)。
