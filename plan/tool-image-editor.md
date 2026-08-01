# 图片编辑器工具

> 状态：**已实现（v0.4.0）** · 入口：首页卡片 → `/tool/image-editor` · 编辑窗口：`image-editor` WebView

## 功能概览

用户可在工具页上传多张本地图片，或打开已保存项目，在**独立全屏编辑窗口**中进行视图缩放、框选裁剪与多模式导出；支持链式裁剪、多图切换、项目持久化。

**已实现：**

- 工具页多图上传（点击 / 拖拽 / Tauri 路径拖放，`accept="image/*" multiple`）
- 上传队列缩略图条：切换预览、追加、删除
- **已保存项目**列表：打开、删除；编辑窗口保存后自动刷新
- 「开始编辑」→ 打开独立窗口（`label: image-editor`，占满当前显示器工作区）
- **多图会话**：编辑窗口内切换图片、追加、删除（至少保留一张）
- **视图缩放**：滚轮 / `+` `-` / 适应窗口 / 可编辑缩放百分比，等比缩放与平移（复用 `picker-canvas.ts`）
- **框选形状**：正方形、矩形、圆形、椭圆
- **选区交互**：拖拽新建、移动、八向调整手柄、清除选区
- **链式裁剪**：「确认裁剪」在工作图上继续叠加；`originalCanvas` 始终保留
- **重置为原图**：`cropCount` 归零，工作图恢复上传内容
- **导出三种模式**：通用单文件 / 缩略图批量 / ICO 批量
- **项目保存**：原图 + 工作图 + 选区状态 + 导出设置，写入 `app_data/image-editor/projects/`
- 入口页 `ToolEntryLayout` 三区布局；编辑区对齐取色器 `picker-session.vue`（左画布 + 右 300px 面板）
- 公共组件：`VoidButton`、`VoidToast`（`useToast`）

**产品约定（已确认）：**

| 问题 | 决策 |
|------|------|
| 裁剪后是否替换原图 | **保留原图**；工作图可链式裁剪，随时重置 |
| 是否支持链式裁剪 | **支持** |
| 是否支持多图 | **支持**；入口队列 + 编辑会话内多文档 |
| 项目持久化 | **支持**；JSON 存于应用数据目录，含编辑状态快照 |
| 导出目标 | **用户选择**；通用模式用另存为对话框，批量模式选目录 |
| 缩略图 / ICO | 要求**正方形**画布或正方形选区（可自动按选区裁剪导出） |
| 导出格式 | PNG / JPEG / WebP / BMP / GIF / TIFF / ICO / AVIF |
| 最大图片尺寸 | **不限制**；大图导出走分块 IPC（>512KB） |
| SVG | 可拖入读取，编辑链路为**栅格**（Canvas + PNG 快照），不支持矢量导出 |

**不在规划内（不做）：**

- 旋转、翻转、画笔、滤镜、文字叠加
- 完整撤销 / 重做栈（仅「重置为原图」与选区清除）
- 从剪贴板 / 截屏直接导入（仅文件上传 / 拖放路径）

---

## 用户流程

### 工具页（`/tool/image-editor` · `index.vue`）

1. 首页点击「图片编辑器」进入工具页（`ToolEntryLayout`）
2. 点击上传区、拖拽文件，或 Tauri 下拖入文件路径（多选）
3. 上传队列显示缩略图，点击切换大预览，可追加或删除
4. 可选：从「已保存的项目」打开历史项目
5. 点击「开始编辑」→ 主窗口通过 IPC 批量写入会话 → 打开独立编辑窗口

### 编辑窗口（`session.vue` + `editor-session.vue`）

1. 窗口启动后 `take_image_editor_session` 取回批量数据，加载多图文档
2. 左侧画布显示当前工作图；顶栏缩略图条切换文档，可追加 / 删除图片
3. **左键拖拽**新建选区；选区可**移动**、**八向缩放**；实时遮罩与尺寸信息
4. **滚轮**缩放视图 · **中键/右键拖拽**平移
5. 选择形状后重新框选；「确认裁剪」更新工作图，可继续链式裁剪
6. 「恢复」重置为原图；「清除选区」移除当前选区
7. **导出设置**（三种 Tab）：
   - **通用**：缩放比例 / 自定义宽高（可锁定等比）、格式、压缩 → 另存为单文件
   - **缩略图**：多选尺寸（16–1024）→ 批量导出至所选目录，命名 `名称_尺寸.ext`
   - **ICO**：多选尺寸（≤256）→ 批量导出 `.ico` 至所选目录
8. 「导出」执行当前模式；Toast 成功后可「在文件夹中显示」
9. 「保存」→ 输入项目名称 → 写入本地项目；主窗口列表自动刷新
10. 「退出编辑」→ 有未保存更改时弹窗：保存并退出 / 直接退出 / 取消
11. **Esc** 触发退出请求（同上逻辑）

---

## UI 图标（Lucide）

| 位置 | 图标 | 文件 |
|------|------|------|
| 工具卡片 | `Image` | `registry.ts` |
| 返回 | `ChevronLeft` | `ToolEntryLayout.vue` |
| 上传区 | `ImagePlus` / `Loader2` | `index.vue` |
| 项目列表 | `FolderOpen` / `Trash2` | `index.vue` |
| 编辑标题 | `Image` | `editor-session.vue` |
| 追加图片 | `ImagePlus` | `editor-session.vue` |
| 确认裁剪 | `Scissors` | `editor-session.vue` |
| 重置为原图 | `RotateCcw` | `editor-session.vue` |
| 清除选区 | `CircleX` | `editor-session.vue` |
| 缩放 | `Plus` / `Minus` / `Scan` | `editor-session.vue` |
| 等比锁定 | `Link2` / `Unlink` | `editor-session.vue` |
| 恢复文件名 | `Undo2` | `editor-session.vue` |
| 保存 | `Save` | `editor-session.vue` |
| 退出编辑 | `X` | `editor-session.vue` |

按钮统一使用 `VoidButton`（`kind`: `button` | `icon`，`size`: `small` | `compact` | `medium` | `large` | `xlarge`）。

---

## 工具页布局

```
← 返回          [Image] 图片编辑器
────────────────────────────────────
┌─ 上传区（flex:1）──────────────────┐
│  [大图预览 / ImagePlus 占位]         │
│  文件名 · 1920 × 1080              │
└────────────────────────────────────┘

已上传 N 张  [thumb][thumb][+] 

📁 已保存的项目
  [预览] 项目名称 · 3 张 · 2026-07-08 14:30  [删除]

────────────────────────────────────
[        开始编辑（VoidButton xlarge）    ]
```

---

## 编辑窗口布局

```
┌─ 画布视口（flex:1）──────────────────────┬─ 操作面板 300px ─────────────┐
│  [Canvas 工作图 + 选区遮罩 + 调整手柄]    │ 图片编辑                      │
│                                          │ photo.png                     │
│                                          │ [+ 点击或拖拽上传图片]         │
│                                          │ [doc thumb][doc thumb]…       │
│                                          │ 选区形状 [正方形][矩形]…      │
│                                          │ 视图缩放  - 100% + 适应       │
│                                          │ ┌─ 导出设置 ─────────────────┐│
│                                          │ │ [通用][缩略图][ICO]        ││
│                                          │ │ 导出名称 · 格式 · 压缩…    ││
│                                          │ └────────────────────────────┘│
│                                          │ [✂][↺][✕]  [    导出    ]   │
│                                          │ [    保存    ][  退出编辑  ] │
└──────────────────────────────────────────┴──────────────────────────────┘
```

**键盘快捷键（会话内）：**

| 键 | 作用 |
|----|------|
| `Esc` | 请求退出（可能弹出保存确认） |
| `+` / `-` | 放大 / 缩小视图 |
| `0` | 适应窗口 |

---

## 架构

```
index.vue ──┬── useImageEditor.ts ──┬── api/image-editor.ts
            │                       │         ↓ invoke
            │                       ├── session batch / project / window
            │                       └── export (path + chunked buffer)
            │
            └── ToolEntryLayout + VoidToast

openImageEditorWindow()
        ↓
App.vue (label=image-editor) → router → session.vue
        ↓
editor-session.vue
        ├── picker-canvas.ts         （缩放 / 平移 / 坐标映射）
        ├── image-editor-crop.ts     （选区几何 / 裁剪 / 移动 / 缩放）
        ├── image-editor-document.ts （编辑状态 PNG 快照）
        ├── image-editor-export.ts   （格式 / 压缩 / 三种导出模式）
        └── image-editor-project.ts  （项目 JSON / 正方形校验）
```

**窗口协作：**

| 窗口 | label | 职责 |
|------|-------|------|
| 主窗口 | `main` | 工具页、项目列表、发起编辑会话 |
| 编辑窗口 | `image-editor` | 全屏编辑；`App.vue` 检测 label 后路由至 `session.vue` |

**会话传递（主窗口 → 编辑窗口）：**

1. `begin_editor_session(sessionId, meta)`
2. `append_editor_session_image` × N
3. `commit_editor_session(sessionId)`
4. `open_image_editor_window()`
5. 编辑窗口 `onMounted` → `take_image_editor_session()` 一次性取回并清空

**三 Canvas 设计（每张文档）：**

| Canvas | 职责 |
|--------|------|
| `originalCanvas` | 离屏，上传原图，**永不修改** |
| `workingCanvas` | 离屏，当前工作图；链式裁剪在此叠加 |
| `viewCanvas` | 可见，按 DPR 绘制缩放/平移后的视图 |

选区遮罩为 DOM overlay（`box-shadow` 镂空 + 形状 `border-radius` + 调整手柄），坐标由 `transform` 映射。

**编辑状态快照（`ImageEditState`）：**

```ts
interface ImageEditState {
  originalPng: string      // 原图 PNG base64
  workingPng: string       // 工作图 PNG base64
  cropCount: number
  width: number
  height: number
  originalWidth: number
  originalHeight: number
  selectionShape: SelectionShape
  selection: SelectionRect | null
}
```

切换文档前自动 `captureEditState`；恢复项目时 `restoreEditState` 写回 Canvas。

---

## 选区与裁剪（`image-editor-crop.ts`）

### 形状约束

| 形状 | 拖拽行为 | 裁剪方式 |
|------|----------|----------|
| 正方形 | `size = max(|dx|, |dy|)` | `drawImage` 矩形子区域 |
| 矩形 | 自由 `dx/dy` | `drawImage` 矩形子区域 |
| 圆形 | 同正方形外接框 | `arc` + `clip` + `drawImage` |
| 椭圆 | 自由矩形外接框 | `ellipse` + `clip` + `drawImage` |

### 交互

- `normalizeSelection`：指针起止点 → 画布内选区；拖拽 < 1px 无效
- `translateSelection` / `resizeSelection`：移动与八向手柄调整
- `clampSelectionToCanvas`：限制在画布边界内

```ts
type SelectionShape = 'square' | 'rect' | 'circle' | 'ellipse'

interface SelectionRect {
  x: number
  y: number
  width: number
  height: number
}
```

---

## 导出（`image-editor-export.ts` + `api/image-editor.ts` + Rust）

### 三种模式

| 模式 | 交互 | 输出 |
|------|------|------|
| `general` | `save` 对话框另存为 | 单文件，可按比例缩放尺寸 |
| `thumbnail` | 选择目录 + 多选正方形尺寸 | `名称_64.png` 等批量文件 |
| `ico` | 选择目录 + 多选 ICO 尺寸（≤256） | `名称_32.ico` 等批量文件 |

缩略图 / ICO 模式要求工作图为正方形，或存在**正方形选区**（导出时自动裁剪选区内容）。

### 支持格式

| 格式 | 编码路径 |
|------|----------|
| PNG | Canvas `toBlob` → 直写或转码 |
| JPEG / WebP | Canvas `toBlob`（质量由压缩档位决定） |
| BMP / GIF / TIFF / ICO / AVIF | `getImageData` 或 PNG 中转 → Rust `image` crate |

压缩档位：100% / 80% / 60% / 50% / 30% / 20%。尺寸缩放档位同上（相对当前画布）。

### 大图分块导出

- 单次 IPC 上限 `EXPORT_SINGLE_SHOT_MAX_BYTES` = 512KB
- 超出时：`begin_export_buffer` → `append_export_base64` × N → `finish_export_binary` / `finish_convert_export`

### IPC API

| Command | 说明 |
|---------|------|
| `read_image_file(path)` | 从路径读取图片元数据与字节 |
| `begin_editor_session` / `append_editor_session_image` / `commit_editor_session` | 批量写入待打开会话 |
| `take_image_editor_session` | 编辑窗口取回并清空会话 |
| `open_image_editor_window` | 创建/替换 `image-editor` 窗口 |
| `save_image_editor_project(json)` | 保存项目 JSON |
| `list_image_editor_projects` | 列出项目摘要 |
| `load_image_editor_project(id)` | 读取项目 JSON |
| `delete_image_editor_project(id)` | 删除项目文件 |
| `export_binary_base64_to_path` | 二进制写入指定路径 |
| `convert_image_base64_to_path` | RGBA/图像转码写入路径 |
| `begin_export_buffer` / `append_export_base64` / `finish_*` | 分块导出 |
| `reveal_export_path(path)` | 在文件管理器中定位文件 |

---

## 项目持久化

**存储路径：** `{app_data}/image-editor/projects/{id}.json`

**项目结构（`ImageEditorProject`）：**

- 元数据：`id`、`name`、`updatedAt`、`previewBase64`
- 会话：`activeDocumentId`、导出模式与各项设置
- 文档数组：每张图含 `sourceBase64`（原图）、`editState`（含 `workingPng`）、`exportName` 等

保存后 `emit('image-editor-projects-changed')`，主窗口 `useImageEditor` 监听并刷新列表。

---

## 前端 Toast（`useToast` + `VoidToast`）

| 场景 | 行为 |
|------|------|
| 成功提示 | 自动关闭（默认 2.5s） |
| 成功 + 「在文件夹中显示」 | 较长展示（8s） |
| 错误提示 | 自动关闭 |
| 鼠标悬停 | 暂停计时；移出后恢复 |

---

## 安全与权限

- Tauri ACL：`src-tauri/permissions/image-editor.toml`
- Capabilities：
  - `src-tauri/capabilities/default.json`（主窗口 `allow-image-editor`）
  - `src-tauri/capabilities/image-editor.json`（编辑窗口专用）
- 无外部网络；图片仅在内存 / 本地上传 / 用户项目目录
- 导出与项目路径经 Rust 侧校验；项目 ID 仅允许安全字符

---

## 测试计划

| 类型 | 内容 |
|------|------|
| Vitest | `normalizeSelection` 几何与边界（`image-editor-crop.test.ts`） |
| Vitest | 导出文件名 / base64 分块（`image-editor-export.test.ts`） |
| 手动 | 多图上传与队列 · 项目保存/打开/删除 · 四形状框选 · 选区移动/缩放 · 链式裁剪 3 次 · 三种导出模式 · 正方形 ICO 提示 · 大图导出 · 退出保存对话框 |

---

## 文件清单

| 路径 | 职责 |
|------|------|
| `plan/tool-image-editor.md` | 本文档 |
| `src/tools/image-editor/index.vue` | 工具入口：多图上传、项目列表、启动编辑 |
| `src/tools/image-editor/session.vue` | 独立窗口壳：多文档状态、保存/导出/退出对话框 |
| `src/tools/image-editor/editor-session.vue` | 编辑 UI：画布、选区、导出设置、工具栏 |
| `src/composables/useImageEditor.ts` | 入口页状态、项目列表、打开编辑窗口 |
| `src/composables/useToast.ts` | Toast 控制器 |
| `src/components/ToolEntryLayout.vue` | 工具入口标准布局 |
| `src/components/VoidButton.vue` | 标准按钮 |
| `src/components/VoidToast.vue` | Toast 展示 |
| `src/utils/image-editor-crop.ts` | 选区几何与 Canvas 裁剪 |
| `src/utils/image-editor-crop.test.ts` | 选区几何单元测试 |
| `src/utils/image-editor-export.ts` | 导出格式、压缩、三种模式 |
| `src/utils/image-editor-export.test.ts` | 导出工具单元测试 |
| `src/utils/image-editor-document.ts` | 多图文档与编辑状态快照 |
| `src/utils/image-editor-project.ts` | 项目 JSON 类型与工具函数 |
| `src/utils/image-file-load.ts` | 文件/路径读取、元数据 |
| `src/api/image-editor.ts` | typed invoke、导出编排、会话批量 |
| `src/utils/picker-canvas.ts` | 复用：缩放、平移、坐标映射 |
| `src/App.vue` | 检测 `image-editor` 窗口并路由至 session |
| `src/router/index.ts` | `/tool/image-editor/session` 路由 |
| `src/tools/registry.ts` | 注册 `Image` 图标 |
| `src-tauri/src/image_editor/mod.rs` | 模块入口 |
| `src-tauri/src/image_editor/session.rs` | 会话状态、读盘 |
| `src-tauri/src/image_editor/project.rs` | 项目 CRUD |
| `src-tauri/src/image_editor/window.rs` | 打开编辑窗口 |
| `src-tauri/src/image_editor/export_buffer.rs` | 分块导出缓冲 |
| `src-tauri/src/export.rs` | 二进制/图像写入、reveal |
| `src-tauri/src/commands.rs` | IPC 命令注册 |
| `src-tauri/permissions/image-editor.toml` | ACL |
| `src-tauri/capabilities/image-editor.json` | 编辑窗口 capability |
| `src-tauri/Cargo.toml` | `image` crate（png/jpeg/webp/gif/bmp/tiff/ico/avif） |

---

## 构建

```bash
pnpm tauri:dev      # 开发调试（含独立编辑窗口）
pnpm build          # 前端类型检查 + 构建
pnpm test           # Vitest（crop + export）
cargo check --manifest-path src-tauri/Cargo.toml
```
