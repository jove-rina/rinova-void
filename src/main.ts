/**
 * main.ts
 * Void 前端应用入口
 *
 * 挂载流程：创建 Vue 应用 → 注册路由 → 挂载到 #app。
 * 全局样式在 App 组件之前引入，确保 CSS 变量与基础排版生效。
 */
import { createApp } from 'vue'
import './styles/main.less'
import router from './router'
import App from './App.vue'

createApp(App).use(router).mount('#app')
