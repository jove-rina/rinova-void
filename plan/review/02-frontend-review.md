# 前端审查

> Round 7 · [13-review-round7-phase3-closed.md](./13-review-round7-phase3-closed.md)

## 状态（Round 7）

| 项 | 状态 |
|----|------|
| registry + Vue Router | ✅ |
| `useClashService` composable | ✅ |
| `api/clash-service.ts` typed invoke | ✅ |
| 首页 Clash running badge | ✅ |
| running 表单 disabled | ✅ |
| success/error 分通道 | ✅ |
| runner 标签（内置/Node） | ✅ Phase 3 |
| URL/端口 localStorage | ✅ |
| 端口占用提示 + 释放/换端口 UI | ✅ |
| 窗口标题栏拖拽 | ✅ |

## Clash 工具 UX 评分

**4.5 / 5**

> 最后验证：2026-07-05 Round 3 · `pnpm build` ✅

## 入口

```typescript
// main.ts
createApp(App).use(router).mount('#app')
```

- Vue Router 已接入 ✅
- 仍无全局错误边界、Tauri 环境检测

## 路由

`src/router/index.ts`：

| 路径 | 组件 | 加载 |
|------|------|------|
| `/` | Home | 同步 |
| `/tool/clash-service` | clash-service | 懒加载 ✅ |

**问题 R3-4**：路由与 `registry.ts` 重复维护。

## 组件审查

### App.vue

| 项 | 状态 |
|----|------|
| scoped 样式 | ✅ |
| router-view | ✅ |
| 窗口关闭 | ✅ |
| 标题双击回首页 | ✅ `goHome()` |
| 窗口遮罩 init | ❌ 注释待实现 |

### WindowHeader.vue

| 项 | 状态 |
|----|------|
| drag-region | ✅ |
| close + aria-label + no-drag | ✅ |
| 标题 @dblclick | ✅ 回首页 |

### Home.vue

| 项 | 状态 |
|----|------|
| registry 驱动 | ✅ |
| 卡片 @click | ✅ `openTool(route)` |
| 图标/箭头 UI | ✅ |
| `--void-accent` 标题 | ✅ |

### clash-service/index.vue（Round 6）

| 项 | 状态 |
|----|------|
| success/error 分通道 | ✅ |
| port/url/base_url 恢复 | ✅ |
| 复制/刷新/Enter | ✅ |
| running 时 URL 仍可编辑 | ⚠️ R6-04 |
| 首页 badge | ❌ |

UX **4.1/5** — [12-review-round6.md](./12-review-round6.md)

## 工具注册表

```typescript
// registry.ts
export interface ToolDefinition {
  id, name, description, icon, route
}
```

建议扩展：`component: () => import(...)` 以驱动路由。

## Vite / TS

- `@/` 别名：配置 ✅，源码已用 ✅
- strictPort / clearScreen / envPrefix ✅

## 依赖

| 包 | 用途 |
|----|------|
| vue-router | 路由 |
| @tauri-apps/api | window + invoke |
| @rinova/proxy-sdk | 仅 proxy-server.mjs 使用（非前端 bundle） |

## 前端评分（Round 3）

| 维度 | R2 | R3 |
|------|----|----|
| 代码质量 | 3.5 | **4** |
| 组件设计 | 3 | **4** |
| 可扩展性 | 2 | **3.5** |
| 无障碍 | 4 | **4** |
| 工程配置 | 5 | **5** |
| 测试覆盖 | 1 | **1** |
