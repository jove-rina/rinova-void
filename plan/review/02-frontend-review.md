# 前端审查

> 最后更新：2026-07-05 · 箭头函数 / JSDoc / Lucide 全量替换

## 状态（当前）

| 项 | 状态 |
|----|------|
| registry + Vue Router（registry 驱动） | ✅ |
| `useClashService` composable | ✅ |
| `api/clash-service.ts` typed invoke | ✅ |
| 首页 Clash running badge | ✅ |
| running 表单 disabled | ✅ |
| success/error 分通道 | ✅ |
| runner 标签（内置/Node） | ✅ |
| URL/端口 localStorage | ✅ |
| 端口占用提示 + 释放/换端口 UI | ✅ |
| 窗口标题栏拖拽 + `init_window` | ✅ |
| **箭头函数规范**（全 src） | ✅ |
| **JSDoc / 模块注释** | ✅ |
| **Lucide 图标（@lucide/vue）** | ✅ |
| registry `icon: LucideIcon` | ✅ |
| Vitest clash-prefs | ✅ 4 tests |

## Clash 工具 UX 评分

**4.5 / 5**

> 验证：`pnpm test` ✅ · `pnpm build` ✅

## 入口

```typescript
// main.ts — 挂载 Vue + Router，引入全局 Less
createApp(App).use(router).mount('#app')
```

- Vue Router 已接入 ✅
- 仍无全局错误边界（低优先级）

## 路由

`src/router/index.ts`：

| 路径 | 组件 | 加载 |
|------|------|------|
| `/` | Home | 同步 |
| `/tool/clash-service` | clash-service | registry 懒加载 ✅ |

路由由 `tools.map` 自动生成，**无需双处维护** ✅

## 代码规范

| 规范 | 说明 |
|------|------|
| 函数 | 全部 `const fn = () => {}`，含 `export const useClashService = () =>` |
| 注释 | 文件头说明 + 接口/函数 JSDoc + 模板关键区块（中文） |
| 图标 | `@lucide/vue` 组件；禁止 emoji、内联 SVG 作 UI 图标 |
| 状态 | 工具页逻辑在 composable，`.vue` 负责展示 |

## Lucide 图标一览

| 场景 | 图标 | 文件 |
|------|------|------|
| 品牌 / favicon | `CircleDot` | `Home.vue`, `public/favicon.svg` |
| Clash 工具卡 | `Shield` | `registry.ts` |
| 列表箭头 | `ChevronRight` | `Home.vue` |
| 关闭 | `X` | `WindowHeader.vue` |
| 返回 | `ChevronLeft` | `clash-service/index.vue` |
| 启停 | `Play` / `Square` | `clash-service/index.vue` |
| 加载 | `Loader2` + spin | `clash-service/index.vue` |
| 复制 | `Copy` / `Check` | `clash-service/index.vue` |
| 刷新 | `RefreshCw` + spin | `clash-service/index.vue` |

新增工具：在 registry 中 `import { FooIcon } from '@lucide/vue'` 并赋给 `icon`。

## 组件审查

### App.vue

| 项 | 状态 |
|----|------|
| scoped 样式 | ✅ |
| router-view | ✅ |
| 窗口关闭（箭头函数） | ✅ |
| 标题双击回首页 | ✅ |
| `init_window` onMounted | ✅ |

### WindowHeader.vue

| 项 | 状态 |
|----|------|
| drag-region + startDragging | ✅ |
| Lucide `X` + aria-label + no-drag | ✅ |
| 标题 @dblclick 回首页 | ✅ |

### Home.vue

| 项 | 状态 |
|----|------|
| registry 驱动 | ✅ |
| 卡片 @click `openTool` | ✅ |
| Lucide 品牌 + 工具图标 + ChevronRight | ✅ |
| 运行中 badge 轮询 | ✅ 5s |

### clash-service/index.vue

| 项 | 状态 |
|----|------|
| `useClashService` 委托 | ✅ |
| success/error 分通道 | ✅ |
| 复制/刷新/Enter | ✅ |
| running 时表单 disabled | ✅ |
| Lucide 操作图标 | ✅ |
| 首页 badge（经 `isClashServiceRunning`） | ✅ |

## 工具注册表

```typescript
// registry.ts
import { Shield, type LucideIcon } from '@lucide/vue'

export interface ToolDefinition {
  id: string
  name: string
  description: string
  icon: LucideIcon          // Lucide 组件，非 emoji 字符串
  route: string
  component: () => Promise<{ default: Component }>
}
```

首页渲染：`<component :is="tool.icon" :size="24" />`

## Vite / TS

- `@/` 别名 ✅
- strictPort / clearScreen / envPrefix ✅
- `vue-tsc -b` 构建前类型检查 ✅

## 依赖

| 包 | 用途 |
|----|------|
| vue / vue-router | UI + 路由 |
| @lucide/vue | 图标组件 |
| @tauri-apps/api | window + invoke |
| @tauri-apps/plugin-window-state | 窗口位置记忆 |
| @tauri-apps/plugin-global-shortcut | 全局快捷键（Rust 侧注册） |
| @rinova/proxy-sdk | 仅 proxy-server 脚本（非前端 bundle） |

## 前端评分

| 维度 | 分数 | 说明 |
|------|------|------|
| 代码质量 | **4.5** | 箭头函数 + JSDoc + api/composable 分层 |
| 组件设计 | **4.5** | registry 驱动、Lucide 统一 |
| 可扩展性 | **4** | 新工具只改 registry + 新目录 |
| 无障碍 | **4** | aria-label、role=alert/status |
| 工程配置 | **5** | Vite 8 + strict TS |
| 测试覆盖 | **2** | clash-prefs 有 Vitest；UI 无 E2E |

## 待改进（Phase 4）

- 全局 Vue error handler
- 组件/E2E 测试
- i18n、主题切换
- 更多工具验证 registry 扩展性
