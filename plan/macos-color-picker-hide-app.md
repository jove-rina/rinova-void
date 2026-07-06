# macOS 取色器：「截屏时隐藏应用」实现路径与经验

> 状态：**已落地**（v0.3.x）  
> 关联：`plan/tool-color-picker.md` · 代码 `src-tauri/src/color_picker/macos/`

---

## 1. 要解决什么问题

取色器在点击「开始取色」时会先截一张屏幕快照，再在全屏会话里从快照上采样颜色。工具页有选项：

**☑ 截屏时隐藏应用窗口**（默认开启，`hideApp` / `hide_app`）

期望行为：

1. 截屏瞬间用户看不到 Void 主窗口（含「正在截屏…」loading）
2. 快照里**不能**出现 Void 自己的 UI
3. 在 **dev** 与 **release `.app`** 上行为一致
4. 在 macOS Sequoia + 屏幕录制 TCC 下可稳定授权

实际难点在于：macOS 上至少有 **三种不同的截屏 API**，各自对「排除本应用窗口」的支持不同；再叠加 **TCC 权限**、**代码签名**、**compositor 刷新时序**，很容易在修 A 问题时踩坏 B。

---

## 2. 最终方案（先看结论）

### 2.1 截屏前：三层隐藏

在 AppKit 主线程执行（`macos/mod.rs` → `hide_window.rs`）：

| 步骤 | API | 作用 |
|------|-----|------|
| 1 | `NSWindowSharingType::None` | 窗口不参与屏幕共享/窗口列表捕获 |
| 2 | `NSWindow.orderOut` | 立即从屏幕 compositor 移除 |
| 3 | `WebviewWindow.hide()` | Tauri 层隐藏 |

然后：

- 首次截屏 sleep **350ms**；会话内 refresh sleep **550ms**
- 主线程 `CFRunLoopRunInMode` 再 flush **120ms**，等 compositor 真正刷新

> **不要** `NSApplication.hide` 整个 App，也**不要**在 hide 时移动窗口到屏外——会污染 `window-state` 插件持久化的坐标。

### 2.2 截屏时：双路径 + 回退

当 `hide_app = true` 时，`capture_display` 走：

```
1. CGWindowListCreateImageFromArray（排除本进程 PID 的全部窗口）  ← 首选，能真正「抠掉」本应用
2. 若返回空 → CGDisplayCreateImage（整屏帧缓冲）                ← 回退，依赖上面 hide 已生效
```

当 `hide_app = false` 时，直接 `CGDisplayCreateImage`。

### 2.3 权限与签名（Sequoia 前置条件）

| 项 | 说明 |
|----|------|
| `Info.plist` | `NSScreenCaptureUsageDescription`（`src-tauri/Info.plist`，Tauri 打包时自动 merge） |
| TCC 预检 | `CGPreflightScreenCaptureAccess` / `CGRequestScreenCaptureAccess`（`screen_access.rs`） |
| Dev 签名 | `tauri.macos.conf.json` → `scripts/macos-dev-runner.sh`：build 后用 Apple Development 证书签二进制 |
| Release | `tauri build` 时 runner 须识别 `build` 子命令（不能只处理 `run`） |

**没有稳定签名时**：TCC 可能显示已授权，但截屏 API 仍返回空——这不是「隐藏应用」逻辑 bug，而是 macOS 把每次编译当成新二进制。

---

## 3. 调用链（从按钮到像素）

```
前端 handleStartPick
  → invoke start_picker(hideApp=true)
    → capture_snapshot(hide_app=true)
      → begin_hide_for_capture          // 主线程 hide + sleep + runloop flush
      → take_platform_snapshot(exclude_own_windows=true)
        → capture_monitor / capture_virtual_desktop
          → capture_display(exclude=true)
            → capture_region (window list) 或 CGDisplay 回退
      → finish_hide_for_capture (Drop guard)
    → layout_picker_window + 返回 PNG base64
```

关键映射：**`hide_app` ≡ `exclude_own_windows`**（macOS 上同一个 bool 贯穿 capture 层）。

---

## 4. 走过的弯路（按时间线）

### 弯路 A：以为 `NSWindowSharingType::None` 对 CGDisplay 有效

**现象**：改为优先 `CGDisplayCreateImage` 后，release 版「截屏 API 返回空」消失，授权也正常了，但**快照里仍能看到 Void 窗口**。

**原因**：

- `NSWindowSharingType::None` 只影响 **窗口列表类 API**（`CGWindowListCreateImage*`）
- `CGDisplayCreateImage` 读的是**整屏帧缓冲**，不管 sharingType

**教训**：注释里写「CGDisplay 帧缓冲无法排除窗口」是对的；不能为了修 API 返回空就 permanently 弃用窗口列表。

---

### 弯路 B：只走窗口列表 API

**现象**：快照不再包含应用，但 release `.app` 在**已授权**时仍报「截屏失败，请确认已允许屏幕录制权限」。

**原因**：

- `CGWindowListCreateImageFromArray` 在 release / 部分系统版本上**偶发返回 null**，与 TCC「预检通过」不矛盾
- 我们把所有 `capture_api_failed_message` 和 `validate_capture_size` 失败都改成了「权限未开」类文案，**误导读用户**

**教训**：

1. **`CGPreflightScreenCaptureAccess() == true` ≠ 窗口列表 API 一定成功**
2. 尺寸校验失败 ≠ 权限失败，应分开提示
3. 需要 **try window list → fallback CGDisplay** 的混合策略

---

### 弯路 C：隐藏时序不足

**现象**：即使用 CGDisplay 回退，偶发仍拍到窗口残影。

**修复**：

- 增加 `orderOut`（原先刻意不用，怕 `show` 恢复不了；实际 `orderFrontRegardless` + `show` 可恢复）
- 拉长 hide 等待 + 主线程 RunLoop flush

---

### 弯路 D：与「隐藏应用」无关但同期踩坑

| 问题 | 原因 | 修复 |
|------|------|------|
| `pnpm tauri:build` → `cargo build build` | `macos-dev-runner.sh` 只剥离了 `run` 没剥离 `build` | runner 同时处理 `run` / `build` |
| `currentMonitor()` TS 报错 | Tauri 2 改为独立函数 | `import { currentMonitor } from '@tauri-apps/api/window'` |
| Dev 每次重编译 TCC 失效 | debug 二进制未签名，Sequoia 当新 app | dev runner 自动 Apple Development 签名 |

---

## 5. macOS 截屏 API 对照表

| API | 需屏幕录制 TCC | 可排除本应用窗口 | release 稳定性 | 适用场景 |
|-----|----------------|------------------|----------------|----------|
| `CGDisplayCreateImage` | 是 | **否**（整屏帧缓冲） | 高 | `hide_app=false`；或 hide 后的回退 |
| `CGWindowListCreateImage` | 是 | 否（含所有 on-screen 窗口） | 高 | 不推荐用于 hide_app |
| `CGWindowListCreateImageFromArray` | 是 | **是**（按 window ID 合成） | 中（偶发 null） | `hide_app=true` 首选 |

辅助手段：

| 手段 | 对 CGDisplay | 对 WindowList |
|------|-------------|---------------|
| `window.hide()` | 依赖 compositor 时序 | 配合 PID 排除更稳 |
| `orderOut` | 帮助从帧缓冲消失 | 同左 |
| `NSWindowSharingType::None` | **无效** | 有效 |

---

## 6. 错误提示设计（用户可见 vs 开发者）

### 用户 Toast（简短）

| 条件 | 文案 |
|------|------|
| TCC 未授权 | 需要屏幕录制权限。请在「系统设置 → 隐私与安全性 → 屏幕录制」中允许 Void，完全退出后重开。 |
| 已授权但 API 失败 | 截屏失败，请重试；若仍失败，可关闭「截屏时隐藏应用」后再试。 |
| 截图像素尺寸异常 | 截屏失败（W×H），请重试或关闭「截屏时隐藏应用」。 |

### 开发者排查（不要写进 Toast）

- `截屏失败：排除本应用后无可用窗口层` → 屏幕上只有 Void，无其他窗口/桌面可合成
- `validate_capture_size` 触发 → 对比 Tauri `monitor.size()` 与 CGImage 宽高，查 DisplayInfo 匹配
- `CGPreflight` false 但系统设置已勾选 → 签名 / 二进制路径变化 / 需完全退出重开

---

## 7. 代码地图

| 文件 | 职责 |
|------|------|
| `macos/hide_window.rs` | sharingType + orderOut + hide；present 时 orderFrontRegardless |
| `macos/mod.rs` | hide 时序、主线程派发、RunLoop flush |
| `macos/capture.rs` | 窗口列表 / CGDisplay 双路径、显示器匹配、像素转换 |
| `macos/screen_access.rs` | TCC 预检与用户文案 |
| `capture.rs` | 跨平台编排；macOS `MacCaptureHideScope` RAII 恢复 |
| `scripts/macos-dev-runner.sh` | dev/build 后 codesign |
| `tauri.macos.conf.json` | 注册 runner |
| `Info.plist` | 屏幕录制用途说明 |

---

## 8. 调试清单

### 8.1 快照里仍有 Void

- [ ] 是否 `hide_app=true`？
- [ ] 窗口列表路径是否成功（若直接 CGDisplay 回退，检查 hide 时序）
- [ ] 增大 `CAPTURE_HIDE_MS` 或 compositor flush 时间试验
- [ ] 截屏时是否还有其他本进程窗口（未来若加多窗口需一并排除）

### 8.2 已授权仍截屏失败

- [ ] 看报错是 TCC 还是 API 失败（不要只看「权限」字样）
- [ ] 关闭「截屏时隐藏应用」试：若成功 → 窗口列表 API 问题，检查回退路径
- [ ] Release：是否 ad-hoc 签名？`codesign -dv path/to/Void.app`
- [ ] Dev：runner 是否输出「已签名: Apple Development」？
- [ ] 改 binary 后：`tccutil reset ScreenCapture com.rinova.void` 再授权

### 8.3 Dev vs Release 行为不一致

- [ ] 比较是否同一签名身份
- [ ] runner 是否在 `tauri build` 路径也执行了 sign
- [ ] `.app` bundle 内实际执行的 Mach-O 路径

---

## 9. 设计原则（后续改动请遵守）

1. **`hide_app` 场景永远不要只依赖 CGDisplay** 来排除本应用——它做不到。
2. **窗口列表失败时必须可回退**，但不能把回退当成主路径；回退依赖 hide 时序。
3. **权限预检通过 ≠ 截屏成功**；用户文案与日志要区分。
4. **NSWindow API 必须在主线程**；hide/show 通过 `run_on_main_thread` 同步。
5. **hide 时不要动窗口几何**；会话布局由 `macos/layout.rs` 在 present 阶段处理。
6. **Sequoia 开发链路**：Xcode 登录 Apple ID → dev runner 签名 → 系统设置授权 → 完全退出重开。

---

## 10. 可选后续优化（未做）

- ScreenCaptureKit（macOS 12.3+）排除当前 app — API 更重，可作为 window list 的长期替代
- 截屏失败时自动重试一次 window list（间隔 + 二次 flush）
- Debug 构建下 `log::debug!` 输出实际走的 capture 路径（window list / CGDisplay）与尺寸

---

## 11. 相关构建命令

```bash
# 开发（macOS 自动走签名 runner）
pnpm tauri:dev

# 本地 release 包
pnpm tauri:build

# 重置屏幕录制授权（改签名 / bundle id 后）
tccutil reset ScreenCapture com.rinova.void
```

---

## 12. 相关文档

| 文档 | 说明 |
|------|------|
| [tool-color-picker.md](./tool-color-picker.md) | 取色器功能规格、IPC、模块结构、Toast 行为 |
| [ARCHITECTURE.zh-CN.md](../ARCHITECTURE.zh-CN.md) | 项目架构与 IPC 总览 |
| [README.zh-CN.md](../README.zh-CN.md) | 用户向快速上手与 macOS 开发备注 |
