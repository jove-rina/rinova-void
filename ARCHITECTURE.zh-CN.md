# 架构说明

Void 代码库的技术概览 — 面向贡献者与维护者。

**语言：** [English](ARCHITECTURE.md) · 简体中文

面向用户的功能与用法见 [README.zh-CN.md](README.zh-CN.md)。

---

## 总览

Void 是基于 **Tauri 2** 的桌面应用，前端为 **Vue 3**。主进程负责系统集成（托盘、快捷键、窗口生命周期）与重计算任务（Clash 代理、屏幕截屏）；WebView 负责 UI 与本地持久化（`localStorage`）。

```
┌─────────────────────────────────────────────────────────────┐
│  Vue 3 前端 (src/)                                          │
│  views · tools · composables · api · utils                  │
└──────────────────────────┬──────────────────────────────────┘
                           │ Tauri invoke (IPC)
┌──────────────────────────▼──────────────────────────────────┐
│  Rust 后端 (src-tauri/src/)                                 │
│  commands · clash · color_picker · tray · window · shortcut │
└─────────────────────────────────────────────────────────────┘
```

---

## 项目结构

```
rinova-void/
├── src/
│   ├── api/              类型化 Tauri invoke 封装（箭头函数 + JSDoc）
│   ├── composables/      共享 Vue composable（useClashService、useColorPicker 等）
│   ├── utils/            工具函数（clash-prefs、color-records、color-format 等）
│   ├── components/       共享 UI（WindowHeader、AboutDialog）
│   ├── views/            页面（Home）
│   ├── tools/            工具模块 + registry.ts
│   ├── router/           Vue Router（由 registry 自动生成路由）
│   └── styles/           全局 Less + CSS 变量主题
├── public/
│   └── favicon.svg       Lucide CircleDot 品牌图标
├── src-tauri/
│   ├── permissions/      Tauri ACL 权限配置
│   ├── icons/            应用打包图标
│   └── src/
│       ├── lib.rs        应用入口、插件初始化、invoke 注册
│       ├── commands.rs   IPC 命令层（薄封装）
│       ├── clash.rs      Clash / rinova-proxy-sdk 集成
│       ├── color_picker/  屏幕截屏与取色会话（Windows GDI · macOS CGDisplay）
│       │   ├── platform/  windows · macos · unsupported
│       │   └── macos/     hide、layout、TCC、窗口列表截屏
│       ├── export.rs     写入 Downloads 文本文件
│       ├── tools.rs      托盘菜单工具列表（与前端 registry 对应）
│       ├── tray.rs       系统托盘图标与菜单
│       ├── window.rs     主窗口初始化、显示/隐藏、工具深链
│       └── shortcut.rs   全局快捷键注册
└── plan/                 工具规格与审查文档
```

---

## 工具注册表

工具在 **`src/tools/registry.ts`** 中统一注册。

每条记录包含 `id`、`name`、`description`、Lucide `icon`、`route` 与懒加载 `component`。路由（`src/router/index.ts`）由该数组展开；首页卡片亦读取同一列表。

**新增工具步骤**

1. 在 `src/tools/<tool-id>/` 下实现页面（及子组件）
2. 在 `registry.ts` 的 `tools[]` 中追加 `ToolDefinition`
3. 若需出现在托盘菜单，在 `src-tauri/src/tools.rs` 中同步添加
4. 在 `commands.rs` + `lib.rs` invoke handler 中注册新 Tauri 命令
5. 如需新权限，在 `src-tauri/permissions/` 下添加 ACL 配置

工具逻辑放在 **composable** 中；页面仅负责布局与事件绑定。

---

## 前端约定

| 约定 | 说明 |
|------|------|
| 函数风格 | 全部使用箭头函数（`const fn = () => {}`），含 composable 与 API 导出 |
| 注释 | 模块头 + 类型/函数 JSDoc + 关键模板区块注释（中文） |
| 图标 | [**@lucide/vue**](https://lucide.dev) 组件，禁止 emoji / 内联 SVG 作 UI 图标 |
| 工具注册 | `registry.ts` 中 `icon` 为 `LucideIcon`；首页 `<component :is="tool.icon" />` |
| 路由 | `createMemoryHistory` — 无地址栏；通过 `router.push` 导航 |
| 状态 | Composable 持有工具状态；用户偏好与记录用 `localStorage` |

---

## IPC 层

`src/api/` 中的模块封装 `invoke()`，提供类型化的参数与返回值。

| 命令 | 模块 | 后端 |
|------|------|------|
| `start_service`、`stop_service`、`get_service_status`、`refresh_service` | `api/clash-service.ts` | `clash.rs` |
| `check_port`、`reclaim_port` | `api/clash-service.ts` | `clash.rs` |
| `list_picker_monitors`、`start_picker`、`refresh_picker`、`finish_picker` | `api/color-picker.ts` | `color_picker/` |
| `export_text_file`、`reveal_export_path` | `api/export.ts` | `export.rs` |
| `init_window` | — | `window.rs` |

典型数据流：

```
工具页 → composable → api/*.ts → commands.rs → 领域模块
```

---

## 后端模块

### Clash（`clash.rs`）

- 在 Tauri 主进程内嵌 [`rinova-proxy-sdk`](https://crates.io/crates/rinova-proxy-sdk)
- 在可配置本地端口暴露 `/clash.yaml`、`/health`、`/refresh`
- 端口回收：对占用端口探测 `/health` 以识别遗留 Void 实例
- 服务状态由 Tauri 管理的 `ClashServiceState` 持有

### 取色器（`color_picker/`）

- **Windows** — GDI 截屏 → PNG base64 → 前端 Canvas 采样
- **macOS** — 窗口列表合成（排除本进程）+ CGDisplay 回退；可选截屏前 hide；work area 布局（非原生全屏）
- 支持单显示器与虚拟桌面（全部屏幕）截屏
- 取色会话在主窗口内；无独立 Overlay WebView
- macOS 需屏幕录制 TCC；开发构建通过 `scripts/macos-dev-runner.sh` 稳定签名 — 见 [plan/macos-color-picker-hide-app.md](plan/macos-color-picker-hide-app.md)
- 窗口关闭、应用退出或显式取消时自动清理

### 托盘与窗口（`tray.rs`、`window.rs`）

- 关闭按钮隐藏主窗口（`prevent_close` + `hide()`）
- 托盘左键切换可见性；菜单提供工具快捷入口与退出
- `open_tool(id)` 显示窗口、触发前端导航，并可排队自动启动（取色器）
- 托盘退出时同步停止 Clash 再退出进程

### 快捷键（`shortcut.rs`）

- 通过 `tauri-plugin-global-shortcut` 注册平台相关切换快捷键

---

## 数据持久化

| 键 / 位置 | 内容 |
|-----------|------|
| `void.clash.prefs`（`localStorage`） | Clash 订阅 URL、端口 |
| `void.color.records`（`localStorage`） | 取色记录（最多 1,000 条） |
| Tauri window-state 插件 | 主窗口位置/尺寸 |

Rust 侧 Clash 状态（当前 URL、运行端口）在进程内存中，通过 `get_service_status` 查询。

---

## 技术栈

| 层级 | 选型 |
|------|------|
| 框架 | Vue 3（Composition API，`<script setup>`） |
| 图标 | @lucide/vue |
| 构建 | Vite 8 |
| 样式 | Less + CSS 变量 |
| 语言 | TypeScript（strict） |
| 桌面 | Tauri 2（无边框，系统托盘） |
| 插件 | window-state、global-shortcut、log（debug） |
| Clash 代理 | rinova-proxy-sdk（Rust，进程内） |
| 屏幕截屏 | Windows GDI · macOS CoreGraphics + 窗口列表（`color_picker/`） |
| 剪贴板 | arboard（Rust，后端按需使用） |
| 路由 | Vue Router（`createMemoryHistory`，registry 驱动） |
| 测试 | Vitest（前端 utils）、cargo test（Rust） |

---

## 发布与 CI

### 本地构建

```bash
pnpm tauri:build
```

产物位于 `src-tauri/target/release/bundle/`。

### GitHub Actions

- **CI**（`.github/workflows/ci.yml`）— Vitest、`cargo test`、前端类型检查/构建、Ubuntu / macOS / Windows 上的 `cargo check`；启用 Rust 构建缓存
- **Release**（`.github/workflows/release.yml`）— 推送 tag `v*`（如 `v0.3.2`）触发；经 `tauri-apps/tauri-action` 构建 macOS Apple Silicon + Intel、Linux、Windows，并发布 GitHub Release。仅当配置了 `APPLE_CERTIFICATE` 时才注入 Apple 签名环境变量，否则 macOS 产出未签名包。

**发布清单**

1. 同步 `package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml` 版本号
2. 更新 `CHANGELOG.md` / `CHANGELOG.zh-CN.md`
3. 合并到 `main` 后打 tag 并推送：`git tag v0.3.2 && git push origin v0.3.2`

| Secret | 用途 |
|--------|------|
| `APPLE_CERTIFICATE` | Base64 `.p12`（macOS） |
| `APPLE_CERTIFICATE_PASSWORD` | 证书密码 |
| `APPLE_SIGNING_IDENTITY` | 如 `Developer ID Application: …` |
| `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` | 公证（可选） |

仓库 **Settings → Actions → General → Workflow permissions** 需设为 **Read and write**，否则无法上传 Release 产物。

### Windows 开发说明

`vite.config.ts` 将 `src-tauri/**` 排除在文件监听之外，避免 `tauri dev` 重编译 `app_lib.dll` 时出现 `EBUSY`。

---

## 生产环境调试

发布版与 `pnpm tauri:dev` 行为不同（CSP 严格生效、无 Vite HMR、Rust 优化编译）。排查安装包问题时建议使用以下方式。

### Debug 安装包（推荐）

构建可安装的 **debug** 包 — 布局与 release 相同，但带调试符号与 `debug_assertions`：

```bash
pnpm tauri:build:debug
```

产物：`src-tauri/target/debug/bundle/`（Windows 为 `.msi`）。

### 文件日志

启动前设置 `VOID_LOG=1`，Rust 日志写入 OS 日志目录：

| 平台 | 路径 |
|------|------|
| Windows | `%LOCALAPPDATA%\com.rinova.void\logs\void.log` |
| macOS | `~/Library/Logs/com.rinova.void/void.log` |
| Linux | `~/.local/share/com.rinova.void/logs/void.log` |

PowerShell 示例：

```powershell
$env:VOID_LOG = "1"
& "C:\Program Files\Void\Void.exe"
```

### 生产环境 DevTools

设置 `VOID_DEVTOOLS=1` 后启动应用，按 **Ctrl+Shift+Alt+I** 打开 WebView DevTools（Console / Network / 断点）。

```powershell
$env:VOID_DEVTOOLS = "1"
$env:VOID_LOG = "1"
& "C:\Program Files\Void\Void.exe"
```

仅当环境变量开启时才注册 DevTools 快捷键，普通用户不受影响。

### 常见 release 特有问题

| 现象 | 原因 |
|------|------|
| 快照一直加载 | CSP 拦截 `data:` 图片 URL（已修复：改用 `createImageBitmap`） |
| 点击取色卡死 | `watch(records)` 与 persist 中重复赋值导致无限 reactive 循环 |
| IPC 静默失败 | ACL 权限缺失 — 用 `VOID_LOG=1` 查看 Rust 日志 |

---

## 安全说明（Clash）

- 订阅 URL 校验与 SSRF 防护在 `clash.rs` 中（见 `cargo test`）
- 本地 HTTP 服务仅绑定 `127.0.0.1`
- CSP 限制前端网络访问为 localhost

各工具的安全细节与边界情况见 `plan/tool-clash-service.md`、`plan/tool-color-picker.md` 与 `plan/macos-color-picker-hide-app.md`。
