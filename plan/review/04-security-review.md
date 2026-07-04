# 安全审查

> 最后更新：2026-07-05 Round 3

## CSP

```json
"csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost; style-src 'self' 'unsafe-inline'; script-src 'self'"
```

仍适用 ✅。Clash 工具通过 Rust 子进程访问外网，**不经过 WebView CSP**。

## Capabilities — Round 3 新风险

自定义 command 未在 `default.json` 声明：

| Command | 敏感度 |
|---------|--------|
| `start_service` | **高** — spawn 进程 + 用户 URL |
| `stop_service` | 中 — kill 进程 |
| `get_service_status` | 低 — 只读 |

**建议**：
- 添加细粒度 permissions
- 仅 `main` 窗口 capability 允许 invoke

## IPC 输入安全

### start_service(url, port)

| 检查项 | 现状 |
|--------|------|
| URL scheme 白名单 | ❌ |
| port 范围 1024-65535 | ❌ Rust 层 |
| 命令注入 | ⚠️ url 作为 argv 传递（非 shell），风险较低 |
| SSRF | ⚠️ 用户 URL 由 SDK 拉取，可访问内网地址 |

**建议**：Rust 校验 `https://` URL；可选 block 私有 IP段。

## 子进程安全

- 依赖本机 `node` 在 PATH — 供应链/劫持风险
- 子进程继承 Tauri 应用环境 — 正常
- piped stdout 未读 — 可用性风险，非直接安全漏洞

## 本地 HTTP 服务

- 绑定 `127.0.0.1`（SDK 默认）— 仅本机访问 ✅
- `/clash.yaml` 暴露代理节点配置 — 预期行为，用户需知悉

## 依赖

| 包 | 风险 |
|----|------|
| @rinova/proxy-sdk | 内部包，需 audit |
| libc | 低 |

仍无 CI audit 流程。

## 安全评分（Round 3）

| 维度 | R2 | R3 |
|------|----|----|
| CSP | 4 | 4 |
| 权限最小化 | 5 | **3** ↓ command 未授权声明 |
| IPC 输入校验 | N/A | **2.5** |
| 本地服务暴露 | — | **4** localhost only |
| **整体** | 4 | **3.5** |

## 发布前清单（更新）

- [x] CSP
- [ ] 自定义 command permissions
- [ ] start_service URL/port 校验
- [ ] cargo audit / pnpm audit
- [ ] 文档说明本地 HTTP 端点安全含义
