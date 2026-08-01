/**
 * router/index.ts
 * Void 应用 Vue Router 配置
 */
import { createRouter, createWebHashHistory } from 'vue-router'
import { tools } from '@/tools/registry'
import Home from '@/views/Home.vue'
import ColorPickerSession from '@/tools/color-picker/session.vue'
import ImageEditorSession from '@/tools/image-editor/session.vue'

/** 将注册表中的工具定义映射为 Vue Router RouteRecord */
const toolRoutes = tools.map((t) => ({
  path: t.route,
  name: t.id,
  component: t.component,
}))

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: Home,
    },
    ...toolRoutes,
    {
      path: '/tool/color-picker/session',
      name: 'color-picker-session',
      component: ColorPickerSession,
    },
    {
      path: '/tool/image-editor/session',
      name: 'image-editor-session',
      component: ImageEditorSession,
    },
  ],
})

export default router
