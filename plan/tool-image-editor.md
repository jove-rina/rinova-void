# 图片编辑器工具

> 状态：**已实现** · 入口：首页卡片 → `/tool/image-editor`

## 功能概览

用户上传本地图片，在全屏编辑会话中进行视图缩放与框选裁剪，支持链式多次裁剪；原图始终保留，可随时重置。

**已实现：**

- 工具页点击 / 拖拽上传图片（`accept="image/*"`）
- 「开始编辑」→ 主窗口内全屏编辑会话（`position: fixed; z-index: 10000`）
- **视图缩放**：滚轮 / `+` `-` / 适应窗口，等比缩放与平移（复用 `picker-canvas.ts`）
- **框选形状**：正方形、矩形、圆形、椭圆
- **链式裁剪**：「应用裁剪」在当前工作图上继续裁剪，可多次叠加
- **保留原图**：`originalCanvas` 不变；「重置为原图」恢复初始上传内容
- **导出**：PNG / JPEG / WebP / BMP / GIF / TIFF / ICO / AVIF，保存至系统下载目录
- 操作面板布局与取色器 `picker-session.vue` 对齐（左侧画布 + 右侧 300px 面板）
- 底部 Toast 提示（成功 / 错误 / 「在文件夹中显示」操作按钮）

**产品约定（已确认）：**

| 问题 | 决策 |
|------|------|
| 裁剪后是否替换原图 | **保留原图**；工作图可链式裁剪，随时重置 |
| 是否支持链式裁剪 | **支持** |
| 导出格式 | **全部常见图片格式**（见下表） |
| 最大图片尺寸 | **不限制** |

**不在规划内（不做）：**

- 旋转、翻转、画笔、滤镜、文字叠加
- 批量处理多张图片
- 撤销 / 重做栈（仅「重置为原图」）
- 从剪贴板 / 截屏直接导入（仅文件上传）

---

## 用户流程

### 工具页（`/tool/image-editor`）

1. 首页点击「图片编辑器」进入工具页
2. 点击上传区或拖拽图片文件
3. 显示文件名与原始尺寸
4. 点击「开始编辑」→ 进入编辑会话

### 编辑会话（`editor-session.vue`）

1. 左侧画布显示当前工作图（初始为上传原图）
2. **左键拖拽**框选区域；实时显示选区信息与遮罩
3. **滚轮**缩放视图 · **中键/右键拖拽**平移
4. 选择形状（正方形 / 矩形 / 圆形 / 椭圆）后重新框选
5. 「应用裁剪」→ 工作图更新为裁剪结果，可继续框选裁剪（链式）
6. 「重置为原图」→ 工作图恢复为上传时的原图
7. 选择导出格式 →「导出当前图片」→ 保存至 Downloads
8. **Esc** 或「退出编辑」返回工具页（原图文件选择仍保留）

---

## UI 图标（Lucide）

| 位置 | 图标 | 文件 |
|------|------|------|
| 工具卡片 | `Image` | `registry.ts` |
| 返回 | `ChevronLeft` | `index.vue` |
| 上传区 | `ImagePlus` / `Loader2` | `index.vue` |
| Toast 成功 | `Check` | `index.vue` |
| 应用裁剪 | `Scissors` | `editor-session.vue` |
| 重置为原图 | `RotateCcw` | `editor-session.vue` |
| 缩放 | `Plus` / `Minus` / `Scan` | `editor-session.vue` |
| 退出编辑 | `X` | `editor-session.vue` |

---

## 工具页布局

```
← 返回          图片编辑器

┌─ 上传区 ─────────────────────────────┐
│         [ImagePlus]                    │
│    点击或拖拽上传图片                   │
│    1920 × 1080 px（已选时显示）         │
└────────────────────────────────────────┘

裁剪后保留原图，可链式多次裁剪；导出支持多种图片格式。

[ 开始编辑 ]
```

---

## 编辑会话布局

```
┌─ 画布视口（flex:1）──────────────────────┬─ 操作面板 300px ─────┐
│                                          │ 图片编辑              │
│  [Canvas 渲染工作图 + 选区遮罩]           │ photo.png · 800×600  │
│                                          │ 左键框选·滚轮缩放...   │
│                                          │ ┌─ 选区形状 ─────────┐ │
│                                          │ │ [正方形][矩形]     │ │
│                                          │ │ [圆形]  [椭圆]     │ │
│                                          │ └───────────────────┘ │
│                                          │ 选区信息 X:… Y:…     │
│                                          │ 已链式裁剪 N 次      │
│                                          │ 视图缩放 - 100% +    │
│                                          │ 导出格式 [PNG ▼]     │
│                                          │ [ 应用裁剪 ]         │
│                                          │ [ 重置为原图 ]       │
│                                          │ [ 导出当前图片 ]     │
│                                          │ [ 退出编辑 ]         │
└──────────────────────────────────────────┴───────────────────────┘
```

**键盘快捷键（会话内）：**

| 键 | 作用 |
|----|------|
| `Esc` | 退出编辑 |
| `+` / `-` | 放大 / 缩小视图 |
| `0` | 适应窗口 |

---

## 架构

```
index.vue ──┬── useImageEditor.ts ── api/image-editor.ts
            │                              ↓ invoke
            └── editor-session.vue         export.rs
                     │
                     ├── picker-canvas.ts      （缩放 / 平移 / 坐标映射）
                     ├── image-editor-crop.ts   （选区几何 / 裁剪）
                     └── image-editor-export.ts （格式列表 / 编码路由）
```

**窗口协作：**

| 窗口 | label | 职责 |
|------|-------|------|
| 主窗口 | `main` | 工具页 + 编辑会话（全屏覆盖层） |

编辑会话在主窗口内以 `EditorSession` 组件呈现，**不使用独立 WebView**。

**三 Canvas 设计：**

| Canvas | 职责 |
|--------|------|
| `originalCanvas` | 离屏，存放上传原图，**永不修改** |
| `workingCanvas` | 离屏，当前编辑工作图；链式裁剪在此叠加 |
| `viewCanvas` | 可见，按 DPR 绘制缩放/平移后的视图 |

选区遮罩为 DOM overlay（`box-shadow` 镂空 + 形状 `border-radius`），坐标由 `transform` 映射。

---

## 选区与裁剪（`image-editor-crop.ts`）

### 形状约束

| 形状 | 拖拽行为 | 裁剪方式 |
|------|----------|----------|
| 正方形 | `size = max(|dx|, |dy|)` | `drawImage` 矩形子区域 |
| 矩形 | 自由 `dx/dy` | `drawImage` 矩形子区域 |
| 圆形 | 同正方形外接框 | `arc` + `clip` + `drawImage` |
| 椭圆 | 自由矩形外接框 | `ellipse` + `clip` + `drawImage` |

### 数据结构

```ts
type SelectionShape = 'square' | 'rect' | 'circle' | 'ellipse'

interface SelectionRect {
  x: number      // 画布像素坐标
  y: number
  width: number
  height: number // 圆/椭圆的外接矩形
}
```

`normalizeSelection(start, end, shape, canvasW, canvasH)` 将指针起止点规范为画布内选区；拖拽距离 < 1px 视为无效。

---

## 导出（`image-editor-export.ts` + Rust）

### 支持格式

| 格式 | 编码路径 |
|------|----------|
| PNG | Canvas `toBlob` → `export_binary_file` |
| JPEG | Canvas `toBlob`（quality 0.92） |
| WebP | Canvas `toBlob`（quality 0.92） |
| BMP / GIF / TIFF / ICO / AVIF | `getImageData` → `export_rgba_image`（Rust `image` crate） |

文件名：`{原文件名去扩展名}-{时间戳}.{ext}`，写入系统下载目录。

### IPC API

| Command | 说明 |
|---------|------|
| `export_binary_file(filename, content)` | 写入任意二进制到 Downloads |
| `export_rgba_image(filename, format, width, height, rgba)` | RGBA 像素转码为指定格式 |
| `reveal_export_path(path)` | 在文件管理器中定位导出文件 |

---

## 前端 Toast（`useImageEditor.ts`）

| 场景 | 自动关闭 |
|------|----------|
| 成功提示 | 2.5s |
| 成功 + 「在文件夹中显示」 | 8s |
| 错误提示 | 2.5s |
| 鼠标悬停在 Toast 上 | 暂停；移出后恢复计时 |

---

## 安全与权限

- Tauri ACL：`src-tauri/permissions/image-editor.toml`
- Capabilities：`src-tauri/capabilities/default.json`（`allow-image-editor`）
- 无外部网络；图片仅在内存 / 本地上传，不自动持久化（除用户主动导出）
- 导出目录为系统 Downloads，文件名经 `sanitize_filename` 过滤 `..`

---

## 测试计划

| 类型 | 内容 |
|------|------|
| Vitest | `normalizeSelection` 矩形/正方形/圆形/边界 clamp（`image-editor-crop.test.ts`） |
| 手动 | 上传 PNG/JPEG · 四种形状框选 · 链式裁剪 3 次 · 重置为原图 · 各格式导出 · 大图（>4K）流畅度 |

---

## 文件清单

| 路径 | 职责 |
|------|------|
| `plan/tool-image-editor.md` | 本文档 |
| `src/tools/image-editor/index.vue` | 工具主页：上传 + 启动编辑 |
| `src/tools/image-editor/editor-session.vue` | 编辑会话：画布 + 操作面板 |
| `src/composables/useImageEditor.ts` | 状态、上传、导出、Toast |
| `src/utils/image-editor-crop.ts` | 选区几何与 Canvas 裁剪 |
| `src/utils/image-editor-crop.test.ts` | 选区几何单元测试 |
| `src/utils/image-editor-export.ts` | 导出格式定义与编码路由 |
| `src/api/image-editor.ts` | typed invoke + 导出封装 |
| `src/utils/picker-canvas.ts` | 复用：缩放、平移、坐标映射 |
| `src/tools/registry.ts` | 注册 `Image` 图标 |
| `src-tauri/src/export.rs` | `export_binary_to_downloads` / `export_rgba_image` |
| `src-tauri/src/commands.rs` | `export_binary_file` / `export_rgba_image` |
| `src-tauri/permissions/image-editor.toml` | ACL |
| `src-tauri/Cargo.toml` | `image` crate 启用 png/jpeg/webp/gif/bmp/tiff/ico/avif |

---

## 构建

```bash
pnpm tauri:dev      # 开发调试
pnpm build          # 前端类型检查 + 构建
pnpm test           # Vitest（含 image-editor-crop）
cargo check --manifest-path src-tauri/Cargo.toml
```
