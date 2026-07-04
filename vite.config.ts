import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],

  // Tauri 推荐：避免清除 Rust 编译输出
  clearScreen: false,

  server: {
    strictPort: true,
    host: host || false,
    port: 5173,
  },

  envPrefix: ['VITE_', 'TAURI_'],

  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
})
