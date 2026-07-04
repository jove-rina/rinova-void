# Void

🕳️ A desktop toolbox — your pocket from the void.

Built with **Tauri 2** + **Vue 3** + **Vite 8** + **Less** + **TypeScript**.

## Prerequisites

- **pnpm** 8+
- **Rust** 1.77+
- **Node.js 18+** — only needed for **development** or if you skip the sidecar build (release bundles a standalone proxy binary)

## Project structure

```
rinova-void/
├── src/
│   ├── api/              Typed Tauri invoke wrappers
│   ├── composables/      Shared Vue composables
│   ├── utils/            clash-prefs 等
│   ├── components/       Shared UI (WindowHeader 拖拽)
│   ├── views/            Pages (Home)
│   ├── tools/            Tool modules (clash-service, …)
│   ├── router/           Vue Router (registry-driven)
│   └── styles/
├── scripts/
│   └── build-sidecar.mjs pkg → Tauri sidecar binary
├── src-tauri/
│   ├── binaries/         proxy-server sidecar (gitignored, built locally)
│   ├── permissions/      Tauri ACL permissions
│   ├── scripts/          proxy-server bundle (Clash tool)
│   └── src/              Rust backend (clash, tray, sidecar)
└── plan/review/          Code review documents
```

## Development

```bash
pnpm install
pnpm tauri:dev       # builds proxy bundle + starts Vite + Tauri
```

Optional — build bundled proxy sidecar (no system Node at runtime):

```bash
pnpm build:sidecar   # ~20s first run (downloads Node base for pkg)
```

Release build (includes sidecar):

```bash
pnpm tauri:build     # runs build:all → .dmg / .msi / …
```

> **macOS build**: if `xcrun` fails:
> ```bash
> DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer pnpm tauri:build
> ```

### Code signing (release)

Set GitHub Actions secrets for signed release builds (`workflow_dispatch` → `release-build` job):

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64 `.p12` (macOS) |
| `APPLE_CERTIFICATE_PASSWORD` | Certificate password |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: …` |
| `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` | Notarization (optional) |

Local unsigned builds work without these secrets.

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
pnpm test:rust     # cargo test (clash SSRF / port scan)
```

## Tools

### Clash 订阅服务

本地 HTTP 服务 exposing `/clash.yaml` for Clash Verge.

- **Prefs**：订阅 URL 与端口自动记忆（重启后恢复）
- **Port**：遗留 Void 进程占端口时自动回收；被其他程序占用时可选手动换端口
- 详见 [plan/tool-clash-service.md](plan/tool-clash-service.md)

## Tech stack

| Layer | Choice |
|-------|--------|
| Framework | Vue 3 (Composition API) |
| Build | Vite 8 |
| Styling | Less |
| Language | TypeScript (strict) |
| Desktop | Tauri 2 (transparent, frameless, system tray) |
