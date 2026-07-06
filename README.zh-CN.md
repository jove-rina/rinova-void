# Void

轻量桌面工具箱 — 空。

基于 **Tauri 2** + **Vue 3** + **TypeScript** 构建。体积小、常驻系统托盘，需要时再唤出。

**支持平台：** macOS（Apple Silicon + Intel）· Windows

**语言：** [English](README.md) · 简体中文

---

## 特色

- **托盘优先** — 关闭窗口即隐藏；Clash 服务可在后台继续运行
- **全局快捷键** — macOS `Cmd+Shift+V` / Windows `Ctrl+Shift+V` 切换显示/隐藏
- **模块化工具** — 每个工具独立成页；通过统一注册表扩展，无需改路由
- **运行时无 Node.js** — 发布版为原生 Tauri 安装包；Node 仅用于开发构建
- **偏好记忆** — 订阅 URL、端口、窗口位置、取色记录等跨重启保留

## 工具

### Clash 订阅服务

为 [Clash Verge](https://github.com/clash-verge-rev/clash-verge-rev) 提供本地 HTTP 订阅端点，无需把机场原始链接直接填入客户端。

| | |
|---|---|
| **端点** | `http://127.0.0.1:{port}/clash.yaml` |
| **运行时** | [`rinova-proxy-sdk`](https://crates.io/crates/rinova-proxy-sdk) 内嵌于主进程（无 sidecar） |
| **自动刷新** | 上游订阅每 60 分钟刷新一次 |
| **偏好** | 订阅 URL 与端口自动保存 |
| **端口** | 自动回收遗留 Void 进程占用的端口；被其他程序占用时可选手动换端口 |

**快速上手**

1. 在首页打开 **Clash 订阅服务**
2. 填入机场订阅 URL，点击 **启动服务**
3. 复制本地地址（如 `http://127.0.0.1:25500/clash.yaml`）
4. 在 Clash Verge 中添加远程订阅，粘贴上述本地地址 — **不要**填机场原始链接

若 Clash 报 `failed to fetch remote profile`，请确认 Void 服务在运行，且浏览器能打开 `/clash.yaml`。

### 取色器

基于屏幕快照精确取色，一次会话可采集多个颜色，并维护持久化色板。

| | |
|---|---|
| **平台** | Windows（GDI）· macOS（屏幕录制权限；截屏时隐藏应用 — 见 [plan/macos-color-picker-hide-app.md](plan/macos-color-picker-hide-app.md)） |
| **会话** | 快照取色，支持缩放、平移与可选像素网格放大镜 |
| **多点取色** | 左键追加颜色；**Esc** 或 **退出取色** 结束会话 |
| **格式** | HEX / RGB / HSL — 可从记录中复制任意格式 |
| **记录** | 最多 1,000 条命名记录（`localStorage`）；重复 HEX 会提示已存在 |
| **导出** | JSON、CSV、Markdown，保存至 Downloads 文件夹 |
| **显示器** | 可选单屏或全部屏幕；PerMonitorV2 DPI 感知 |
| **Toast** | 底部成功/错误提示；鼠标悬停时暂停自动关闭 |
| **托盘入口** | 托盘菜单 → **取色器** 打开工具页并自动开始取色 |

**快速上手**

1. 在首页（或托盘菜单）打开 **取色器**
2. （可选）选择目标显示器、放大倍数、截屏时是否隐藏 Void
3. 点击 **开始取色** → 等待截屏完成 → 在快照上点击像素采集颜色
4. 滚轮缩放、中键/右键拖拽平移；在侧栏管理记录
5. 退出后可在工具页重命名、复制、导出或删除记录

---

## 安装

### 从 GitHub Releases 安装

**macOS**（Apple Silicon + Intel）与 **Windows** 预编译包见 [Releases](https://github.com/jove-rina/rinova-void/releases) 页面（`.dmg` / `.msi`）。

### 从源码（开发 / 本地构建）

**环境要求**

- **pnpm** 8+
- **Rust** 1.77+
- **Node.js 18+** — 仅用于前端开发/构建

```bash
git clone <repo-url> rinova-void
cd rinova-void
pnpm install
pnpm tauri:dev       # 热重载
```

**发布构建**

```bash
pnpm tauri:build     # 输出 .msi / .dmg 等，位于 src-tauri/target/release/bundle/
```

> **macOS：** 若 `xcrun` 失败：
> ```bash
> DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer pnpm tauri:build
> ```

> **macOS 取色器（开发）：** `pnpm tauri:dev` 会通过 `scripts/macos-dev-runner.sh` 用 Apple Development 证书签名，避免 Sequoia 上每次重编译后屏幕录制权限失效。请先在 Xcode 登录 Apple ID。改签名后重置 TCC：`tccutil reset ScreenCapture com.rinova.void`

> **Windows 开发：** Vite 忽略 `src-tauri/**`，避免 Rust 重编译时 `app_lib.dll` 出现 `EBUSY`。

GitHub Actions 签名发布需配置 Apple 证书 Secrets — 详见 [ARCHITECTURE.zh-CN.md](ARCHITECTURE.zh-CN.md#发布与-ci)。本地未签名构建无需配置。

发布新版本：更新版本号与 CHANGELOG，合并到 `main` 后执行 `git tag v0.3.4 && git push origin v0.3.4`。Release workflow 会自动从 `CHANGELOG.md` 提取对应版本内容作为 Release 正文。

---

## 使用说明

### 系统托盘

| 操作 | 效果 |
|------|------|
| 关闭窗口（×） | 隐藏到托盘；后台服务继续运行 |
| 左键托盘图标 | 切换窗口显示/隐藏 |
| 托盘菜单 → 工具名 | 显示窗口并跳转到对应工具 |
| 托盘菜单 → **退出** | 完全退出并停止 Clash |

### 全局快捷键

- **macOS：** `Cmd+Shift+V`
- **Windows：** `Ctrl+Shift+V`

切换主窗口显示/隐藏；下次启动时恢复窗口位置。

### 首页

首页展示所有已注册工具。Clash 服务运行时会显示 **运行中** 徽章（每 5 秒轮询一次）。

---

## 测试

```bash
pnpm test          # Vitest — 前端 utils
pnpm test:rust     # cargo test — clash SSRF / 端口扫描 / 状态辅助
```

---

## 文档

| 文档 | 说明 |
|------|------|
| [ARCHITECTURE.zh-CN.md](ARCHITECTURE.zh-CN.md) | 项目结构、约定、IPC 与后端模块 |
| [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md) | 版本历史 |
| [plan/tool-clash-service.md](plan/tool-clash-service.md) | Clash 工具规格 |
| [plan/tool-color-picker.md](plan/tool-color-picker.md) | 取色器规格 |
| [plan/macos-color-picker-hide-app.md](plan/macos-color-picker-hide-app.md) | macOS「截屏时隐藏应用」实现与踩坑 |

---

## 技术栈（概要）

Vue 3 · Vite 8 · Less · TypeScript · Tauri 2 · Lucide 图标 · rinova-proxy-sdk（Rust）

完整说明见 [ARCHITECTURE.zh-CN.md](ARCHITECTURE.zh-CN.md)。
