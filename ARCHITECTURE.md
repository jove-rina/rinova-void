# Architecture

Technical overview of the Void codebase — for contributors and maintainers.

**Languages:** English · [简体中文](ARCHITECTURE.zh-CN.md)

For user-facing features and usage, see [README.md](README.md).

---

## Overview

Void is a **Tauri 2** desktop app with a **Vue 3** frontend. The main process owns system integration (tray, shortcuts, window lifecycle) and heavy work (Clash proxy, screen capture). The webview handles UI and local persistence (`localStorage`).

```
┌─────────────────────────────────────────────────────────────┐
│  Vue 3 frontend (src/)                                      │
│  views · tools · composables · api · utils                  │
└──────────────────────────┬──────────────────────────────────┘
                           │ Tauri invoke (IPC)
┌──────────────────────────▼──────────────────────────────────┐
│  Rust backend (src-tauri/src/)                              │
│  commands · clash · color_picker · tray · window · shortcut │
└─────────────────────────────────────────────────────────────┘
```

---

## Project structure

```
rinova-void/
├── src/
│   ├── api/              Typed Tauri invoke wrappers (arrow functions + JSDoc)
│   ├── composables/      Shared Vue composables (useClashService, useColorPicker, …)
│   ├── utils/            Helpers (clash-prefs, color-records, color-format, …)
│   ├── components/       Shared UI (WindowHeader, AboutDialog)
│   ├── views/            Pages (Home)
│   ├── tools/            Tool modules + registry.ts
│   ├── router/           Vue Router (routes auto-generated from registry)
│   └── styles/           Global Less + CSS variable theme
├── public/
│   └── favicon.svg       Lucide CircleDot brand icon
├── src-tauri/
│   ├── permissions/      Tauri ACL capability files
│   ├── icons/            App bundle icons
│   └── src/
│       ├── lib.rs        App entry, plugin setup, invoke handler
│       ├── commands.rs   IPC command surface (thin wrappers)
│       ├── clash.rs      Clash / rinova-proxy-sdk integration
│       ├── color_picker/  屏幕截屏与取色会话（Windows GDI · macOS CGDisplay）
│       │   ├── platform/  windows · macos · unsupported
│       │   └── macos/     hide、layout、TCC、窗口列表截屏
│       ├── export.rs     Write text files to Downloads
│       ├── tools.rs      Tray menu tool list (mirror of frontend registry)
│       ├── tray.rs       System tray icon and menu
│       ├── window.rs     Main window init, show/hide, deep-link to tools
│       └── shortcut.rs   Global shortcut registration
└── plan/                 Tool specifications and review notes
```

---

## Tool registry

Tools are registered in a **single source of truth**: `src/tools/registry.ts`.

Each entry defines `id`, `name`, `description`, Lucide `icon`, `route`, and a lazy `component` factory. The router (`src/router/index.ts`) expands this array into routes; the home page renders cards from the same list.

**Adding a tool**

1. Create `src/tools/<tool-id>/index.vue` (and subcomponents as needed)
2. Append a `ToolDefinition` to `tools[]` in `registry.ts`
3. If the tool should appear in the tray menu, add a matching entry in `src-tauri/src/tools.rs`
4. Register any new Tauri commands in `commands.rs` + `lib.rs` invoke handler
5. Add ACL permissions under `src-tauri/permissions/` if required

Tool logic belongs in **composables**; pages handle layout and event binding only.

---

## Frontend conventions

| Convention | Description |
|------------|-------------|
| Function style | Arrow functions only (`const fn = () => {}`), including composables and API exports |
| Comments | Module header + JSDoc on types/functions + key template block comments (Chinese OK in code) |
| Icons | [**@lucide/vue**](https://lucide.dev) components — no emoji or inline SVG for UI icons |
| Tool registry | `icon` in `registry.ts` is a `LucideIcon`; Home uses `<component :is="tool.icon" />` |
| Routing | `createMemoryHistory` — no URL bar; navigation via `router.push` |
| State | Composables own tool state; `localStorage` for user prefs and records |

---

## IPC layer

Frontend API modules in `src/api/` wrap `invoke()` calls with typed arguments and return values.

| Command | Module | Backend |
|---------|--------|---------|
| `start_service`, `stop_service`, `get_service_status`, `refresh_service` | `api/clash-service.ts` | `clash.rs` |
| `check_port`, `reclaim_port` | `api/clash-service.ts` | `clash.rs` |
| `list_picker_monitors`, `start_picker`, `refresh_picker`, `finish_picker` | `api/color-picker.ts` | `color_picker/` |
| `export_text_file`, `reveal_export_path` | `api/export.ts` | `export.rs` |
| `init_window` | — | `window.rs` |

Event flow for tools typically follows:

```
tool page → composable → api/*.ts → commands.rs → domain module
```

---

## Backend modules

### Clash (`clash.rs`)

- Embeds [`rinova-proxy-sdk`](https://crates.io/crates/rinova-proxy-sdk) in the Tauri main process
- Exposes `/clash.yaml`, `/health`, `/refresh` on a configurable local port
- Port reclaim: probes `/health` on occupied ports to detect stale Void instances
- Service state held in `ClashServiceState` managed by Tauri

### Color picker (`color_picker/`)

- **Windows** — GDI screen capture → PNG base64 → frontend canvas sampling
- **macOS** — `CGWindowListCreateImageFromArray` (exclude own windows) with `CGDisplay` fallback; optional hide-before-capture; work-area layout instead of native fullscreen
- Supports per-monitor and virtual-desktop (all screens) capture
- Picker session in main window; no separate overlay WebView
- macOS requires Screen Recording TCC; dev builds use `scripts/macos-dev-runner.sh` for stable signing — see [plan/macos-color-picker-hide-app.md](plan/macos-color-picker-hide-app.md)
- Cleanup on window close, app exit, or explicit cancel

### Tray & window (`tray.rs`, `window.rs`)

- Close button hides the main window (`prevent_close` + `hide()`)
- Tray left-click toggles visibility; menu provides tool shortcuts and quit
- `open_tool(id)` shows window, navigates via frontend event, and can queue auto-start (color picker)
- Quit from tray stops Clash synchronously before exit

### Shortcut (`shortcut.rs`)

- Registers platform-specific toggle shortcut via `tauri-plugin-global-shortcut`

---

## Data persistence

| Key / location | Content |
|----------------|---------|
| `void.clash.prefs` (`localStorage`) | Clash subscription URL, port |
| `void.color.records` (`localStorage`) | Color picker records (max 1,000) |
| Tauri window state plugin | Main window position/size |

Rust-side Clash state (active URL, running port) lives in process memory and is queried via `get_service_status`.

---

## Tech stack

| Layer | Choice |
|-------|--------|
| Framework | Vue 3 (Composition API, `<script setup>`) |
| Icons | @lucide/vue |
| Build | Vite 8 |
| Styling | Less + CSS variables |
| Language | TypeScript (strict) |
| Desktop | Tauri 2 (frameless, system tray) |
| Plugins | window-state, global-shortcut, log (debug) |
| Clash proxy | rinova-proxy-sdk (Rust, in-process) |
| Screen capture | Windows GDI · macOS CoreGraphics + window list (`color_picker/`) |
| Clipboard | arboard (Rust, if used by backend) |
| Routing | Vue Router (`createMemoryHistory`, registry-driven) |
| Tests | Vitest (frontend utils), cargo test (Rust) |

---

## Release & CI

### Local build

```bash
pnpm tauri:build
```

Outputs platform bundles under `src-tauri/target/release/bundle/`.

### GitHub Actions

- **CI** (`.github/workflows/ci.yml`) — Vitest, `cargo test`, frontend typecheck/build, `cargo check` on Ubuntu, macOS, and Windows; Rust build cache enabled. Ubuntu jobs run `scripts/ci-linux-deps.sh` (WebKit/GTK/glib per [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)) before any Rust step.
- **Release** (`.github/workflows/release.yml`) — triggered by pushing tag `v*` (e.g. `v0.3.3`); builds macOS Apple Silicon + Intel, Linux (`ubuntu-22.04`, `libwebkit2gtk-4.0-dev`), and Windows via `tauri-apps/tauri-action`, then publishes a GitHub Release. Linux deps via the same `scripts/ci-linux-deps.sh` with distro-specific WebKit package. Apple signing secrets are injected only when `APPLE_CERTIFICATE` is configured (checked in a shell script, not step `if:`); otherwise macOS builds unsigned.

**Release checklist**

1. Bump version in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`
2. Update `CHANGELOG.md` / `CHANGELOG.zh-CN.md`
3. Merge to `main`, then tag and push: `git tag v0.3.3 && git push origin v0.3.3`

| Secret | Purpose |
|--------|---------|
| `APPLE_CERTIFICATE` | Base64 `.p12` (macOS) |
| `APPLE_CERTIFICATE_PASSWORD` | Certificate password |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: …` |
| `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` | Notarization (optional) |

Repository **Settings → Actions → General → Workflow permissions** must allow **Read and write** for release uploads.

### Windows dev note

`vite.config.ts` excludes `src-tauri/**` from file watching to prevent `EBUSY` errors when `app_lib.dll` is rebuilt during `tauri dev`.

---

## Production debugging

Release builds differ from `pnpm tauri:dev` (CSP enforcement, no Vite HMR, optimized Rust). Use these workflows to reproduce and inspect production issues.

### Debug bundle (recommended)

Build an installable **debug** package — same layout as release, but with debug symbols and `debug_assertions`:

```bash
pnpm tauri:build:debug
```

Output: `src-tauri/target/debug/bundle/` (`.msi` on Windows).

### File logging

Set `VOID_LOG=1` before launching the installed app. Rust logs are written to the OS log directory:

| Platform | Path |
|----------|------|
| Windows | `%LOCALAPPDATA%\com.rinova.void\logs\void.log` |
| macOS | `~/Library/Logs/com.rinova.void/void.log` |
| Linux | `~/.local/share/com.rinova.void/logs/void.log` |

PowerShell example:

```powershell
$env:VOID_LOG = "1"
& "C:\Program Files\Void\Void.exe"
```

### DevTools in production

Set `VOID_DEVTOOLS=1`, launch the app, then press **Ctrl+Shift+Alt+I** to open WebView DevTools (Console / Network / breakpoints).

```powershell
$env:VOID_DEVTOOLS = "1"
$env:VOID_LOG = "1"
& "C:\Program Files\Void\Void.exe"
```

DevTools are only registered when the env var is set — normal users are unaffected.

### Typical release-only pitfalls

| Symptom | Cause |
|---------|-------|
| Snapshot stuck on loading | CSP blocked `data:` image URLs (fixed: `createImageBitmap`) |
| Freeze on pick | `watch(records)` + reassign in persist handler caused infinite reactive loop |
| IPC silent failure | Missing ACL permission — check Rust log with `VOID_LOG=1` |

---

## Security notes (Clash)

- Subscription URL validation and SSRF guards live in `clash.rs` (see `cargo test`)
- Local HTTP server binds to `127.0.0.1` only
- CSP restricts frontend network access to localhost

Tool-specific security and edge cases are documented in `plan/tool-clash-service.md`, `plan/tool-color-picker.md`, and `plan/macos-color-picker-hide-app.md`.
