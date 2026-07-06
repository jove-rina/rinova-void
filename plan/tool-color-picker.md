# 取色器工具

> 状态：**v0.3.0 已发布** · 入口：首页卡片 / 工具页 / **托盘菜单「取色」**

## 功能概览

从屏幕截取快照，在快照上精确取色，支持一次会话内采集多个颜色，记录持久化到 localStorage。

**已实现：**

- 工具页配置截屏参数 →「开始取色」→ 主窗口全屏进入取色会话
- **截屏快照取色**（Rust 侧 GDI 截屏 → PNG base64 → 前端双 Canvas 采样），非实时 GetPixel 逐帧采样
- **多点取色**：左键点击追加记录，Esc /「退出取色」结束会话
- **可选放大镜网格**（radius 0–8，对应 1×1 至 17×17 像素网格，中心格高亮）
- 取色会话内：**滚轮缩放视图**、**中键/右键拖拽平移**、**刷新屏幕**、**切换显示器**、**截取全部屏幕**
- **PerMonitorV2 DPI** 感知，Canvas 按 `devicePixelRatio` 渲染避免模糊
- **取色记录**：可展开/收起、编辑标题、复制 HEX/RGB/**HSL**、删除、**JSON 导出**；工具页与操作面板共用同一数据源
- **记录策略**：最多 **100** 条（超出丢弃最旧）；**HEX 去重**（重复取色提示已存在）
- **托盘菜单「取色」**：显示窗口 → 跳转工具页 → 自动开始截屏取色
- 截屏 / 刷新时 loading 遮罩；底部 Toast 提示（成功 / 错误）
- 窗口关闭 / 应用退出时自动 `cancel_picker` 清理会话

**不在规划内（不做）：**

- 全局快捷键启动取色
- macOS / Linux 截屏
- 滑块调色板 / 图片导入取色
- 单点取色后自动复制并退出（旧 Overlay 方案）
- Phase 2+ 以外的任何新功能

---

## 用户流程

### 工具页（`/tool/color-picker`）

1. 首页点击「取色器」进入工具页
2. （可选）展开「取色记录」查看 / 管理历史
3. 配置：
   - **目标显示器**（多显示器时可选，含 DPI 比例与主屏标记）
   - **截取全部屏幕**（虚拟桌面合并截屏）
   - **放大倍数**（启动时的默认 radius）
   - **截屏时隐藏应用窗口**
4. 点击「开始取色」→ 显示截屏 loading → 进入取色会话

**记录展开模式：** 有记录且展开时，工具页仅保留「取色记录 + 开始取色」，容器无滚动条，列表内部滚动。

### 取色会话（`picker-session.vue`）

1. 主窗口 `setFullscreen(true)`，左侧为快照视口，右侧 300px 操作面板
2. 鼠标移动实时预览当前像素（HEX / RGB）；开启放大倍数时显示像素网格
3. **左键点击**（移动距离 ≤ 6px 视为点击）→ 追加一条取色记录
4. **滚轮**缩放视图 · **中键/右键拖拽**平移 · **Esc** 或「退出取色」结束
5. 面板内可切换显示器 / 全屏截屏 / 刷新快照 / 调整放大倍数 / 管理记录
6. 退出时 Toast 提示「本次共取 N 个颜色」（N 为本次会话新增数）

---

## UI 图标（Lucide）

| 位置 | 图标 | 文件 |
|------|------|------|
| 工具卡片 | `Pipette` | `registry.ts` |
| 返回 | `ChevronLeft` | `index.vue` |
| 开始取色 / loading | `Pipette` / `Loader2` | `index.vue` |
| 记录展开/收起 | `ChevronUp` / `ChevronDown` | `index.vue` |
| Toast 成功 | `Check` | `index.vue` |
| 记录：编辑/复制/删除 | `Pencil` / `Copy` / `Trash2` | `color-record-list.vue` |
| 会话：刷新/缩放/退出 | `RefreshCw` / `Plus` `Minus` `Scan` / `X` | `picker-session.vue` |

---

## 工具页布局

```
← 返回          取色器

┌─ 取色记录 ──────────── 共 N 条 [展开] ─┐
│  （折叠时仅显示摘要；展开时列表可滚动）   │
└────────────────────────────────────────┘

目标显示器    [ 显示器 1 · 1920×1080 · 主屏 ▼ ]
☑ 截取全部屏幕
放大倍数      [ 关闭 ▼ ]
☑ 截屏时隐藏应用窗口

[ 开始取色 ]
```

**展开记录时：**

```
← 返回          取色器

┌─ 取色记录 ──────────── 共 N 条 [收起] ─┐
│  ■ 颜色 1  #c084fc  rgb(...)  ✎ 📋 🗑  │
│  ■ 颜色 2  ...                        │  ← 内部滚动
│  ...                                  │
└────────────────────────────────────────┘

[ 开始取色 ]                               ← 贴底
```

---

## 取色会话布局

```
┌─ 快照视口（flex:1）──────────────────────┬─ 操作面板 300px ─────┐
│                                          │ 取色                  │
│  [Canvas 渲染截屏，crosshair 光标]        │ 显示器标签             │
│                                          │ 左键取色·滚轮缩放...   │
│                                          │ ┌─ 截屏设置 ─────────┐ │
│                                          │ │ 截取屏幕 / 全屏    │ │
│                                          │ │ [刷新屏幕]         │ │
│                                          │ └───────────────────┘ │
│                                          │ [像素网格 / 色块预览]  │
│                                          │ #c084fc               │
│                                          │ rgb(192, 132, 252)    │
│                                          │ 视图缩放 - 100% +     │
│                                          │ 放大倍数 [5× ▼]       │
│                                          │ ┌─ 取色记录 ─────────┐ │
│                                          │ │ （与工具页同 UI）  │ │ ← 内部滚动
│                                          │ └───────────────────┘ │
│                                          │ [ 退出取色 ]          │
└──────────────────────────────────────────┴───────────────────────┘
```

**键盘快捷键（会话内）：**

| 键 | 作用 |
|----|------|
| `Esc` | 退出取色 |
| `M` | 循环切换放大倍数 |
| `+` / `-` | 放大 / 缩小视图 |
| `0` | 适应窗口 |

---

## 架构

```
index.vue ──┬── useColorPicker.ts ── api/color-picker.ts
            │                              ↓ invoke
            └── picker-session.vue         commands.rs → color_picker.rs
                     │                              ↓
                     ├── picker-canvas.ts      GDI BitBlt 截屏
                     └── color-record-list.vue       ↓
                                              PNG base64 → 前端 Canvas 采样
```

**窗口协作：**

| 窗口 | label | 职责 |
|------|-------|------|
| 主窗口 | `main` | 工具页 + 取色会话（全屏） |

取色会话在主窗口内以 `position: fixed; z-index: 10000` 的 `PickerSession` 组件呈现，**不使用独立 Overlay WebView**（避免 Windows 多窗口问题）。

**双 Canvas 设计（`picker-canvas.ts`）：**

- **sourceCanvas**：离屏，存放原始截图像素，`willReadFrequently: true`，供 `getImageData` 采样
- **viewCanvas**：可见，按 DPR 尺寸绘制缩放/平移后的视图

---

## IPC API

| Command | 说明 |
|---------|------|
| `list_picker_monitors()` | 枚举显示器（index、label、坐标、尺寸、是否主屏） |
| `start_picker(radius?, hideApp?, monitorIndex?, captureAll?)` | 截屏并启动会话，返回 PNG 快照 + 元数据 |
| `refresh_picker(hideApp?, monitorIndex?, captureAll?, radius?)` | 会话中重新截屏，可切换显示器/全屏/radius |
| `finish_picker(cancel, r?, g?, b?)` | 结束会话；当前前端统一传 `cancel=true`（多点模式不写 Rust 剪贴板） |

**`StartPickerResult`：**

```ts
interface StartPickerResult {
  image_base64: string   // PNG
  width: number
  height: number
  origin_x: number       // 截屏区域在虚拟桌面的偏移
  origin_y: number
  radius: number         // 当前放大半径 (0–8)
  monitor_index: number
  monitor_label: string
  capture_all: boolean
}
```

**放大倍数选项（`MAGNIFY_OPTIONS`）：**

| label | radius | 网格 |
|-------|--------|------|
| 关闭 | 0 | 1×1 |
| 3× | 1 | 3×3 |
| 5× | 2 | 5×5 |
| 7× | 3 | 7×7 |
| 9× | 4 | 9×9 |
| 11× | 5 | 11×11 |
| 13× | 6 | 13×13 |
| 17× | 8 | 17×17 |

前端 `gridMeta(radius)` 计算 `gridSize = 2r+1` 与中心索引。

---

## 取色记录持久化

**文件：** `src/utils/color-records.ts`  
**存储键：** `void.color.records`（`localStorage` JSON 数组）

```ts
interface ColorRecord {
  id: string          // crypto.randomUUID()
  name: string        // 默认「颜色 N」，可编辑
  hex: string
  r: number           // 0–255
  g: number
  b: number
  createdAt: number   // Date.now()
}
```

- 加载时校验字段与 RGB 范围，无效条目过滤
- 按 `createdAt` 降序排列；新记录插入头部
- `useColorPicker` 对 `records` 做 `deep watch` 自动保存
- 复制走前端 `navigator.clipboard`（HEX / RGB / HSL 下拉菜单）
- 最多 **100** 条（`MAX_COLOR_RECORDS`），保存与加载时自动裁剪
- **HEX 去重**：同 HEX 再次取色时不新增，Toast 提示「颜色已存在：{name}」
- **导出**：工具页记录区「导出」按钮 → 下载 `void-colors-YYYY-MM-DD.json`

---

## 颜色格式工具（纯 TS）

**文件：** `src/utils/color-format.ts`

| 函数 | 输出示例 |
|------|----------|
| `toHex(r,g,b)` | `#c084fc` |
| `toRgb(r,g,b)` | `rgb(192, 132, 252)` |
| `toHsl(r,g,b)` | `hsl(270, 95%, 75%)` |
| `parseHex(hex)` | `[192, 132, 252]` 或 `null` |

Vitest 覆盖 `toHex` / `toRgb` / `toHsl` / `parseHex`（`color-format.test.ts`）。

---

## 后端实现要点

### Windows（当前唯一实现平台）

- **DPI**：`SetThreadDpiAwarenessContext(PER_MONITOR_AWARE_V2)` + `GetDpiForMonitor`
- **显示器枚举**：`EnumDisplayMonitors` + `GetMonitorInfoW`
- **单屏截屏**：`BitBlt` 从屏幕 DC 复制到 DIB
- **全屏截屏**：虚拟桌面范围合并截取
- **截屏前隐藏主窗口**（可选，`CAPTURE_HIDE_MS = 120ms` 等待）
- **PNG 编码**：`image` crate → base64 返回前端

### 会话状态（`PickerState`）

```rust
struct PickerState {
    is_active: Mutex<bool>,
    session: Mutex<Option<PickerSession>>,  // 持有 ScreenCapture + radius
}
```

- `cancel_picker` / `is_picker_active`：供窗口关闭、应用退出时清理
- `finish_picker(cancel=true)`：释放会话、退出全屏、聚焦主窗口，不写剪贴板
- `finish_picker(cancel=false, r, g, b)`：Rust 侧写剪贴板（保留兼容，前端多点模式未使用）

### 依赖

| Crate | 用途 |
|-------|------|
| `arboard` | 剪贴板（单点 finish 兼容路径） |
| `image` | PNG 编码 |
| `base64` | 快照传输 |

### 非 Windows

`color_picker.rs` 的 `platform` 模块目前仅 `#[cfg(windows)]`，其他平台 invoke 会失败。

---

## 安全与权限

- Tauri ACL：`src-tauri/permissions/color-picker.toml`
- Capabilities：`src-tauri/capabilities/default.json`
- 无外部网络；快照仅内存持有，不持久化到磁盘
- 取色记录仅存 WebView `localStorage`

---

## 与初版方案的主要差异

| 初版（Phase A 文档） | 当前实现 |
|---------------------|----------|
| 独立 Overlay 窗口 + `sample_pixels` 实时采样 | 主窗口全屏 + 截屏快照 + Canvas 采样 |
| 左键确认 → 复制 → 立即退出 | 左键追加记录，多点后手动退出 |
| 固定 11×11 放大镜 | 8 档可选 radius（含关闭） |
| 最近 12 条 HEX 历史 | 无限条命名记录（HEX/RGB/编辑/删除） |
| HEX/RGB/HSL 格式切换 | 仅 HEX + RGB |
| `color-picker-prefs.ts` | `color-records.ts` |
| `overlay.vue` | `picker-session.vue` + `color-record-list.vue` |

---

## 分阶段交付（更新）

### 已完成

- [x] 工具页 + `registry.ts` 注册
- [x] Rust：截屏、`list_picker_monitors` / `start_picker` / `refresh_picker` / `finish_picker`
- [x] 取色会话 UI（快照视口 + 操作面板）
- [x] 多点取色 + 记录持久化
- [x] 多显示器 / 全屏截屏 / 会话内刷新
- [x] Canvas 缩放平移 + 可选放大镜网格
- [x] PerMonitorV2 DPI + 双 Canvas 防模糊
- [x] 记录展开布局、共用 `ColorRecordList` 组件
- [x] Toast 提示、loading 遮罩
- [x] 窗口关闭 / 退出时 cancel 清理
- [x] Vitest（`color-format` / `color-records`）

### Phase 2+（已完成）

- [x] **托盘菜单「取色」** — `tray.rs` 菜单项 → `window::open_color_picker` → `open-color-picker` 事件 → 工具页自动 `start_picker`
- [x] **HSL 格式支持** — `toHsl`；会话预览 + 记录列表展示；复制菜单「复制 HSL」
- [x] **Rust 单元测试** — `rgb_to_hex`；`finish_picker_state(cancel=true)` 清会话且不复制；非活跃/缺 RGB 分支
- [x] **记录策略** — 上限 100 条、JSON 导出、HEX 去重

> 明确不做：全局快捷键、跨平台截屏、调色板、图片取色等未列出的功能。

---

## 测试计划

| 类型 | 内容 |
|------|------|
| Vitest | `toHex` / `toRgb` / `toHsl` / `parseHex`；`color-records` 去重与裁剪 |
| Rust | `rgb_to_hex`；`finish_picker_state` cancel / inactive / 缺 RGB |
| 手动 | 托盘「取色」自动启动 · 复制 HSL · 导出 JSON · HEX 去重 · 100 条上限提示 · （其余见上） |

---

## 文件清单

| 路径 | 职责 |
|------|------|
| `plan/tool-color-picker.md` | 本文档 |
| `src/tools/color-picker/index.vue` | 工具主页：记录区 + 截屏设置 + 启动 |
| `src/tools/color-picker/picker-session.vue` | 取色会话：快照视口 + 操作面板 |
| `src/tools/color-picker/color-record-list.vue` | 取色记录列表（工具页与会话共用） |
| `src/composables/useColorPicker.ts` | 状态、IPC 流程、记录 CRUD、Toast |
| `src/utils/color-records.ts` | 记录 localStorage、上限/去重/导出 |
| `src/utils/color-records.test.ts` | 去重与裁剪单元测试 |
| `src/utils/color-format.ts` | HEX/RGB/HSL 转换 |
| `src/utils/color-picker-launch.ts` | 托盘等外部入口的自动启动队列 |
| `src/utils/picker-canvas.ts` | Canvas 坐标映射、采样、缩放平移 |
| `src/App.vue` | 监听 `open-color-picker` 事件 |
| `src-tauri/src/tray.rs` | 托盘菜单含「取色」 |
| `src-tauri/src/window.rs` | `open_color_picker` 显示窗口并 emit |
| `src/api/color-picker.ts` | typed invoke + `MAGNIFY_OPTIONS` / `gridMeta` |
| `src/tools/registry.ts` | 注册 `Pipette` 图标 |
| `src-tauri/src/color_picker.rs` | 截屏、会话状态、剪贴板 |
| `src-tauri/src/commands.rs` | 4 个 picker command |
| `src-tauri/src/lib.rs` | 窗口事件 / 退出时 cancel_picker |
| `src-tauri/permissions/color-picker.toml` | ACL |

**已移除 / 不再使用：**

- `src/tools/color-picker/overlay.vue`
- `src/utils/color-picker-prefs.ts`
- `picker-overlay` 独立窗口（`tauri.conf.json` 无此窗口）
- IPC `sample_pixels`

---

## 构建

```bash
pnpm tauri:dev      # 开发调试取色
pnpm build          # 前端类型检查 + 构建
pnpm test           # Vitest（color-format / color-records）
cargo test --manifest-path src-tauri/Cargo.toml color_picker  # Rust 取色器测试
```
