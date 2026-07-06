# Changelog

All notable changes to Void are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Languages:** English · [简体中文](CHANGELOG.zh-CN.md)

---

## [Unreleased]

---

## [0.3.0] - 2026-07-06

### Added

- **Color picker** tool — snapshot-based screen picking on Windows
  - Multi-color sessions with zoom, pan, and optional pixel-grid magnifier (radius 0–8)
  - Per-monitor or all-screens capture; DPI-aware canvas rendering
  - Persistent color records (up to 1,000 entries): rename, copy HEX/RGB/HSL, delete
  - Export records as JSON, CSV, or Markdown to Downloads
  - HEX deduplication with toast feedback
  - Tray menu entry with auto-start picking
- Export helpers — `export_text_file` / `reveal_export_path` Tauri commands
- Tray menu lists all registered tools (synced with frontend registry)
- About dialog (version info)

### Changed

- Home screen and window header UI refinements
- Project documentation reorganized — README (user guide), CHANGELOG, and ARCHITECTURE (developer guide), each in English and Chinese

---

## [0.2.0] - 2026-07-06

### Added

- Embedded [`rinova-proxy-sdk`](https://crates.io/crates/rinova-proxy-sdk) — Clash proxy runs in-process in the Tauri main process

### Changed

- **Breaking (internal):** Removed Node.js sidecar and bundled proxy script; no separate proxy process
- Updated app icons and branding assets (CircleDot logo, refreshed platform icons)
- Clash service UI: built-in runner label, improved port handling feedback
- README and tool documentation updates

### Removed

- `scripts/build-sidecar.mjs`, `src-tauri/scripts/proxy-server.*`, `src-tauri/src/sidecar.rs`

---

## [0.1.0] - 2026-07-05

### Added

- Initial release — **Void** desktop toolbox (Tauri 2 + Vue 3)
- **Clash subscription service** — local `/clash.yaml` HTTP endpoint for Clash Verge
  - Start / stop / manual refresh
  - Subscription URL and port persistence
  - Smart port reclaim for stale Void processes; optional port fallback
- System tray — hide on close, toggle via click, quit from menu
- Global shortcut — `Cmd+Shift+V` / `Ctrl+Shift+V` to toggle window
- Window position restore (`tauri-plugin-window-state`)
- Tool registry pattern — home cards and routes driven by `src/tools/registry.ts`
- Frameless main window with custom drag header
- CI — Vitest, Rust unit tests, multi-platform build checks

[Unreleased]: https://github.com/jove-rina/rinova-void/compare/0.3.0...HEAD
[0.3.0]: https://github.com/jove-rina/rinova-void/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/jove-rina/rinova-void/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/jove-rina/rinova-void/releases/tag/0.1.0
