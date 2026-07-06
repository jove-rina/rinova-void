# Void

A desktop toolbox — your pocket from the void.

Built with **Tauri 2** + **Vue 3** + **Vite 8** + **Less** + **TypeScript** + **Lucide** icons.

## Prerequisites

- **pnpm** 8+
- **Rust** 1.77+ (Tauri backend + embedded [rinova-proxy-sdk](https://crates.io/crates/rinova-proxy-sdk))
- **Node.js 18+** — frontend dev/build only (release app does not require Node at runtime)

## Project structure

```
rinova-void/
├── src/
│   ├── api/              Typed Tauri invoke wrappers（箭头函数 + JSDoc）
│   ├── composables/      Shared Vue composables（如 useClashService）
│   ├── utils/            clash-prefs 等工具函数
│   ├── components/       Shared UI（WindowHeader 拖拽/关闭）
│   ├── views/            Pages（Home）
│   ├── tools/            Tool modules + registry.ts（Lucide 图标注册）
│   ├── router/           Vue Router（由 registry 自动生成路由）
│   └── styles/           全局 Less + CSS 变量主题
├── public/
│   └── favicon.svg       Lucide CircleDot 品牌图标
├── src-tauri/
│   ├── permissions/      Tauri ACL permissions
│   └── src/              Rust backend (clash, tray, window, shortcut)
└── plan/                 工具与审查文档
    ├── tool-clash-service.md
    └── review/
```

## Development

```bash
pnpm install
pnpm tauri:dev       # Vite + Tauri hot reload
```

Release build:

```bash
pnpm tauri:build     # pnpm build → Tauri bundle (.msi / .dmg / …)
```

> **macOS build**: if `xcrun` fails:
> ```bash
> DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer pnpm tauri:build
> ```

> **Windows dev**: Vite ignores `src-tauri/**` to avoid `EBUSY` on `app_lib.dll` during Rust rebuilds.

### Code signing (release)

Set GitHub Actions secrets for signed release builds (`workflow_dispatch` → `release-build` job):

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64 `.p12` (macOS) |
| `APPLE_CERTIFICATE_PASSWORD` | Certificate password |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: …` |
| `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` | Notarization (optional) |

Local unsigned builds work without these secrets.

## Frontend conventions

| 约定 | 说明 |
|------|------|
| 函数风格 | 全部使用**箭头函数**（`const fn = () => {}`），含 composable 与 API 导出 |
| 注释 | 模块头 + 类型/函数 JSDoc + 关键模板区块注释（中文） |
| 图标 | [**@lucide/vue**](https://lucide.dev) 组件，禁止 emoji / 内联 SVG 作 UI 图标 |
| 工具注册 | `src/tools/registry.ts` 中 `icon` 为 `LucideIcon` 组件，首页 `<component :is="tool.icon" />` |
| 状态 | 工具逻辑放 composable（`useClashService`），页面只做展示与事件绑定 |

## System tray

- Close the window → **hides to menu bar / system tray** (Clash service keeps running)
- Left-click tray icon → show/hide window
- Tray menu → **退出 Void** to fully quit and stop Clash

## Global shortcut

- **macOS**: `Cmd+Shift+V` · **Windows/Linux**: `Ctrl+Shift+V` — toggle window show/hide

Window position is restored on next launch (`tauri-plugin-window-state`).

## Testing

```bash
pnpm test          # Vitest (frontend utils)
pnpm test:rust     # cargo test (clash SSRF / port scan / status helpers)
```

## Tools

### Clash 订阅服务

本地 HTTP 服务 exposing `/clash.yaml` for Clash Verge.

- **Runtime**：[`rinova-proxy-sdk`](https://crates.io/crates/rinova-proxy-sdk) 内嵌于 Tauri 主进程，无需 Node / sidecar
- **Prefs**：订阅 URL 与端口自动记忆（重启后恢复）
- **Port**：遗留 Void 进程占端口时自动回收（不会 kill 当前进程）；被其他程序占用时可选手动换端口
- **Clash 订阅**：在 Clash Verge 中添加 `http://127.0.0.1:{port}/clash.yaml`，不要直接填机场原始链接
- 详见 [plan/tool-clash-service.md](plan/tool-clash-service.md)

## Tech stack

| Layer | Choice |
|-------|--------|
| Framework | Vue 3 (Composition API, `<script setup>`) |
| Icons | @lucide/vue |
| Build | Vite 8 |
| Styling | Less + CSS variables |
| Language | TypeScript (strict) |
| Desktop | Tauri 2 (frameless, system tray) |
| Clash proxy | rinova-proxy-sdk (Rust, in-process) |
| Routing | Vue Router (`createMemoryHistory`, registry-driven) |
