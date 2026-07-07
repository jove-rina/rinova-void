# 更新日志

Void 的所有重要变更均记录于此。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

**语言：** [English](CHANGELOG.md) · 简体中文

---

## [未发布]

---

## [0.3.4] - 2026-07-07

### 变更

- **平台支持** — 取消 Linux 官方支持；Release 与 CI 仅构建 **macOS** 与 **Windows**。打包目标限定为 `dmg` / `app` / `msi` / `nsis`；移除 Linux CI runner 与 `scripts/ci-linux-deps.sh`
- **GitHub Release** — Release 正文由 `scripts/extract-changelog.sh` 从 `CHANGELOG.md` 提取对应 tag 版本条目（Release 页附简体中文 CHANGELOG 链接）

---

## [0.3.3] - 2026-07-07

### 修复

- **GitHub Release workflow** — Apple 签名配置不再在 step 的 `if:` 中使用 `secrets`（GitHub Actions 不支持）；改为在 shell 脚本内判断证书是否存在
- **跨平台构建** — 取色器：非 macOS 路径导入 `tauri::Manager`（`window_layout.rs`）；`capture.rs` 中 `PickerState` 仅 macOS 分支导入
- **Windows 构建** — `clash.rs` 补充 `std::process::Stdio` 导入，供 `taskkill` 丢弃 stdout/stderr

---

## [0.3.2] - 2026-07-07

### 修复

- **GitHub Release（macOS）** — 仅在配置了 Apple 签名 Secret 时才注入 `APPLE_*` 环境变量；避免 `security import` 失败，未配置开发者证书时可成功产出未签名 macOS 安装包

### 变更

- 取色器截屏 / 窗口布局后端小幅调整

---

## [0.3.1] - 2026-07-07

### 新增

- **macOS 取色器** — 基于屏幕快照取色，需屏幕录制权限；可选「截屏时隐藏应用」，避免快照中出现 Void 界面
- **GitHub Release 自动化** — 推送 tag `v*`（如 `v0.3.1`）自动构建 macOS（Apple Silicon + Intel）与 Windows 并发布 GitHub Release

### 变更

- 取色器后端重构为平台模块（`color_picker/macos/`、`platform/`）
- CI — 构建矩阵增加 Windows；启用 Rust 构建缓存；Release 构建迁移至 `.github/workflows/release.yml`

---

## [0.3.0] - 2026-07-06

### 新增

- **取色器**工具 — Windows 上基于屏幕快照的取色
  - 多点取色会话，支持缩放、平移与可选像素网格放大镜（radius 0–8）
  - 单显示器或全部屏幕截屏；DPI 感知 Canvas 渲染
  - 持久化取色记录（最多 1,000 条）：重命名、复制 HEX/RGB/HSL、删除
  - 导出 JSON、CSV、Markdown 至 Downloads 文件夹
  - HEX 去重并 Toast 提示
  - 托盘菜单入口，自动开始取色
- 导出辅助 — `export_text_file` / `reveal_export_path` Tauri 命令
- 托盘菜单列出所有已注册工具（与前端 registry 同步）
- 关于对话框（版本信息）

### 变更

- 首页与窗口标题栏 UI 优化
- 项目文档重组 — README（用户指南）、CHANGELOG、ARCHITECTURE（开发者指南），均提供中英文版本

---

## [0.2.0] - 2026-07-06

### 新增

- 内嵌 [`rinova-proxy-sdk`](https://crates.io/crates/rinova-proxy-sdk) — Clash 代理在 Tauri 主进程内运行

### 变更

- **破坏性（内部）：** 移除 Node.js sidecar 与打包的 proxy 脚本，不再启动独立代理进程
- 更新应用图标与品牌资源（CircleDot logo、各平台图标）
- Clash 服务 UI：内置运行标签、端口占用反馈改进
- README 与工具文档更新

### 移除

- `scripts/build-sidecar.mjs`、`src-tauri/scripts/proxy-server.*`、`src-tauri/src/sidecar.rs`

---

## [0.1.0] - 2026-07-05

### 新增

- 首次发布 — **Void** 桌面工具箱（Tauri 2 + Vue 3）
- **Clash 订阅服务** — 为 Clash Verge 提供本地 `/clash.yaml` HTTP 端点
  - 启动 / 停止 / 手动刷新
  - 订阅 URL 与端口持久化
  - 遗留 Void 进程端口智能回收；可选自动换端口
- 系统托盘 — 关闭隐藏、点击切换、菜单退出
- 全局快捷键 — `Cmd+Shift+V` / `Ctrl+Shift+V` 切换窗口
- 窗口位置恢复（`tauri-plugin-window-state`）
- 工具注册表模式 — 首页卡片与路由由 `src/tools/registry.ts` 驱动
- 无边框主窗口与自定义拖拽标题栏
- CI — Vitest、Rust 单元测试、多平台构建检查

[未发布]: https://github.com/jove-rina/rinova-void/compare/v0.3.4...HEAD
[0.3.4]: https://github.com/jove-rina/rinova-void/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/jove-rina/rinova-void/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/jove-rina/rinova-void/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/jove-rina/rinova-void/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/jove-rina/rinova-void/compare/0.2.0...v0.3.0
[0.2.0]: https://github.com/jove-rina/rinova-void/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/jove-rina/rinova-void/releases/tag/0.1.0
