/**
 * registry.ts
 * Void 工具注册表 — 单一数据源驱动首页卡片与路由
 *
 * 新增工具步骤：
 * 1. 在 `src/tools/<tool-id>/` 下实现 Vue 页面组件（入口页使用 `ToolEntryLayout`）
 * 2. 在本数组追加一条 ToolDefinition（含 Lucide 图标组件）
 * 3. 操作按钮使用 `VoidButton`（`kind`: button | icon，`size`: small | compact | medium | large | xlarge）
 * 4. 无需修改 router/index.ts（路由由本表自动生成）
 */
import type { Component } from 'vue'
import { Image, Pipette, Shield, type LucideIcon } from '@lucide/vue'

/**
 * 单个 Void 工具的元数据与懒加载入口。
 */
export interface ToolDefinition {
  /** 唯一标识，用于路由 name、首页运行状态 Set 的 key */
  id: string
  /** 首页卡片标题 */
  name: string
  /** 首页卡片副标题/简介 */
  description: string
  /** 首页卡片 Lucide 图标组件 */
  icon: LucideIcon
  /** Vue Router path，建议 `/tool/<id>` 格式 */
  route: string
  /** 懒加载组件工厂，供 Vue Router 动态 import */
  component: () => Promise<{ default: Component }>
}

/**
 * 当前已注册的全部工具列表。
 * 顺序即首页展示顺序。
 */
export const tools: ToolDefinition[] = [
  {
    id: 'clash-service',
    name: 'Clash 订阅服务',
    description: '启动本地 Clash 订阅 HTTP 服务',
    icon: Shield,
    route: '/tool/clash-service',
    component: () => import('@/tools/clash-service/index.vue'),
  },
  {
    id: 'color-picker',
    name: '取色器',
    description: '从屏幕吸取颜色并复制 HEX',
    icon: Pipette,
    route: '/tool/color-picker',
    component: () => import('@/tools/color-picker/index.vue'),
  },
  {
    id: 'image-editor',
    name: '图片编辑器',
    description: '多图裁剪、项目保存与多格式导出',
    icon: Image,
    route: '/tool/image-editor',
    component: () => import('@/tools/image-editor/index.vue'),
  },
]
