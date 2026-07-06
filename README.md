# Void

A lightweight desktop toolbox — your pocket from the void.

Built with **Tauri 2** + **Vue 3** + **TypeScript**. Small footprint, lives in the system tray, and stays out of your way until you need it.

**Languages:** English · [简体中文](README.zh-CN.md)

---

## Highlights

- **Tray-first** — Close the window to hide; Clash service keeps running in the background
- **Global shortcut** — `Cmd+Shift+V` (macOS) / `Ctrl+Shift+V` (Windows/Linux) toggles show/hide
- **Modular tools** — Each tool is a self-contained page; new tools plug in via a single registry
- **No runtime Node.js** — The release app is a native Tauri bundle; Node is only needed for development
- **Remembers your prefs** — Subscription URL, port, window position, and color history persist across restarts

## Tools

### Clash subscription service

Run a local HTTP endpoint for [Clash Verge](https://github.com/clash-verge-rev/clash-verge-rev) — no need to paste your airport URL directly into the client.

| | |
|---|---|
| **Endpoint** | `http://127.0.0.1:{port}/clash.yaml` |
| **Runtime** | [`rinova-proxy-sdk`](https://crates.io/crates/rinova-proxy-sdk) embedded in-process (no sidecar) |
| **Auto-refresh** | Upstream subscription refreshed every 60 minutes |
| **Prefs** | Subscription URL and port are saved automatically |
| **Port handling** | Reclaims ports held by stale Void processes; optional fallback when another app occupies the port |

**Quick start**

1. Open **Clash 订阅服务** from the home screen
2. Paste your airport subscription URL and click **Start**
3. Copy the local address (e.g. `http://127.0.0.1:25500/clash.yaml`)
4. In Clash Verge, add a remote profile with that local URL — **not** the raw airport link

If Clash reports `failed to fetch remote profile`, confirm Void is running and that `/clash.yaml` opens in a browser.

### Color picker

Pick colors from a screen snapshot, collect multiple swatches in one session, and manage a persistent palette.

| | |
|---|---|
| **Platform** | Windows (GDI) · macOS (screen recording permission; hide-app capture — see [plan/macos-color-picker-hide-app.md](plan/macos-color-picker-hide-app.md)) |
| **Session** | Snapshot-based picking with zoom, pan, and optional pixel grid magnifier |
| **Multi-pick** | Left-click to add colors; **Esc** or **Exit** to finish the session |
| **Formats** | HEX / RGB / HSL — copy any format from records |
| **Records** | Up to 1,000 named entries in `localStorage`; duplicate HEX is detected |
| **Export** | JSON, CSV, or Markdown to your Downloads folder |
| **Displays** | Per-monitor or all-screens capture; DPI-aware rendering |
| **Toast** | Bottom toast for success/errors; pauses auto-dismiss while hovered |
| **Tray shortcut** | Tray menu → **取色器** opens the tool and starts picking automatically |

**Quick start**

1. Open **取色器** from the home screen (or use the tray menu)
2. (Optional) Choose target display, magnifier level, and whether to hide Void while capturing
3. Click **开始取色** → wait for the snapshot → click pixels to collect colors
4. Use scroll wheel to zoom, middle/right button to pan; manage records in the side panel
5. After exiting, rename, copy, export, or delete records from the tool page

---

## Installation

### From source (development / local build)

**Prerequisites**

- **pnpm** 8+
- **Rust** 1.77+
- **Node.js 18+** — frontend dev/build only

```bash
git clone <repo-url> rinova-void
cd rinova-void
pnpm install
pnpm tauri:dev       # hot reload
```

**Release build**

```bash
pnpm tauri:build     # outputs .msi / .dmg / etc. under src-tauri/target/release/bundle/
```

> **macOS:** If `xcrun` fails:
> ```bash
> DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer pnpm tauri:build
> ```

> **macOS color picker (dev):** `pnpm tauri:dev` uses `scripts/macos-dev-runner.sh` to sign the binary with an Apple Development certificate so Screen Recording permission persists across rebuilds. Log in to Xcode with your Apple ID first. Reset TCC after signing changes: `tccutil reset ScreenCapture com.rinova.void`

> **Windows dev:** Vite ignores `src-tauri/**` to avoid `EBUSY` on `app_lib.dll` during Rust rebuilds.

Signed release builds via GitHub Actions require Apple signing secrets — see [ARCHITECTURE.md](ARCHITECTURE.md#release--ci) for details. Local unsigned builds work without them.

---

## Usage

### System tray

| Action | Result |
|--------|--------|
| Close window (×) | Hides to tray; background services keep running |
| Left-click tray icon | Toggle window show/hide |
| Tray menu → tool name | Show window and navigate to that tool |
| Tray menu → **退出** | Fully quit and stop Clash |

### Global shortcut

- **macOS:** `Cmd+Shift+V`
- **Windows / Linux:** `Ctrl+Shift+V`

Toggles the main window. Window position is restored on next launch.

### Home screen

The home screen lists all registered tools. A **运行中** badge appears when Clash service is active (polled every 5 seconds).

---

## Testing

```bash
pnpm test          # Vitest — frontend utils
pnpm test:rust     # cargo test — clash SSRF / port scan / status helpers
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Project structure, conventions, IPC, and backend modules |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [plan/tool-clash-service.md](plan/tool-clash-service.md) | Clash tool specification |
| [plan/tool-color-picker.md](plan/tool-color-picker.md) | Color picker specification |
| [plan/macos-color-picker-hide-app.md](plan/macos-color-picker-hide-app.md) | macOS hide-app capture: APIs, pitfalls, debugging |

---

## Tech stack (summary)

Vue 3 · Vite 8 · Less · TypeScript · Tauri 2 · Lucide icons · rinova-proxy-sdk (Rust)

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full breakdown.
