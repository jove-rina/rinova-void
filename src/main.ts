/**
 * main.ts
 * Void 前端应用入口
 */
import { createApp } from 'vue'
import './styles/main.less'
import router from './router'
import App from './App.vue'

createApp(App).use(router).mount('#app')
