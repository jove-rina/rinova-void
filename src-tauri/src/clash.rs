use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use tauri::Manager;

use crate::sidecar::{self, ProxyRunner};

// ─── Types ─────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct ServiceStatus {
    pub status: String,
    pub port: Option<u16>,
    pub url: Option<String>,
    pub base_url: Option<String>,
    #[serde(rename = "runner")]
    pub runner_kind: String,
}

pub struct ClashServiceState {
    pub child: Mutex<Option<Child>>,
    pub active_port: Mutex<Option<u16>>,
    pub active_url: Mutex<Option<String>>,
    pub runner: ProxyRunner,
}

// ─── Node resolution (dev fallback) ────────────────────────

pub fn resolve_node_path() -> String {
    use std::path::Path;

    if node_available("node") {
        return "node".to_string();
    }
    for candidate in [
        "/opt/homebrew/bin/node",
        "/usr/local/bin/node",
        "/usr/bin/node",
    ] {
        if Path::new(candidate).is_file() && node_available(candidate) {
            return candidate.to_string();
        }
    }
    "node".to_string()
}

fn node_available(path: &str) -> bool {
    use std::process::Command;
    Command::new(path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ─── URL validation (SSRF guard) ───────────────────────────

pub fn validate_subscription_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim().to_string();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err("URL 必须以 http:// 或 https:// 开头".to_string());
    }
    if trimmed.len() < 12 {
        return Err("URL 格式无效".to_string());
    }

    let host = trimmed
        .split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .and_then(|h| h.split('@').last())
        .and_then(|h| h.split(':').next())
        .unwrap_or("")
        .to_lowercase();

    if host.is_empty() {
        return Err("URL 格式无效".to_string());
    }

    if is_blocked_host(&host) {
        return Err("不允许使用内网或本地地址作为订阅 URL".to_string());
    }

    Ok(trimmed)
}

fn is_blocked_host(host: &str) -> bool {
    if host == "localhost" || host.ends_with(".localhost") {
        return true;
    }
    if host.starts_with('[') {
        let inner = host.trim_start_matches('[').trim_end_matches(']');
        return inner == "::1"
            || inner.starts_with("fe80:")
            || inner.starts_with("fc")
            || inner.starts_with("fd");
    }
    if host
        .parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback() || is_private_ip(&ip))
        .unwrap_or(false)
    {
        return true;
    }
    false
}

fn is_private_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_private() || v4.is_link_local() || v4.is_broadcast(),
        std::net::IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local(),
    }
}

// ─── Process helpers ───────────────────────────────────────

fn kill_child(child: &mut Child) {
    #[cfg(unix)]
    unsafe {
        libc::kill(child.id() as i32, libc::SIGTERM);
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    for _ in 0..30 {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait();
}

pub fn stop_service_impl(state: &ClashServiceState) {
    if let Ok(mut child_lock) = state.child.lock() {
        if let Some(mut child) = child_lock.take() {
            kill_child(&mut child);
        }
    }
    if let Ok(mut port_lock) = state.active_port.lock() {
        *port_lock = None;
    }
    if let Ok(mut url_lock) = state.active_url.lock() {
        *url_lock = None;
    }
}

fn stderr_temp_path() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("rinova-void-stderr-{}.log", nanos))
}

// ─── HTTP helpers ──────────────────────────────────────────

fn health_check(port: u16) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("地址解析失败: {}", e))?,
        Duration::from_secs(2),
    )
    .map_err(|_| format!("端口 {} 未监听到服务", port))?;

    let req = format!(
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        port
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|_| "发送请求失败".to_string())?;

    let mut buf = [0u8; 256];
    let n = stream
        .read(&mut buf)
        .map_err(|_| "读取响应失败".to_string())?;
    let resp = String::from_utf8_lossy(&buf[..n]);

    if resp.contains("200 OK") || resp.contains("\"status\":\"ok\"") {
        Ok(())
    } else {
        Err("服务响应不符合预期".to_string())
    }
}

fn is_port_listening(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(1)).is_ok()
}

/// Max ports to scan upward when `allow_fallback` is enabled.
const PORT_SCAN_MAX: u16 = 32;

fn wait_port_free(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !is_port_listening(port) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    !is_port_listening(port)
}

fn is_likely_our_proxy(port: u16) -> bool {
    health_check(port).is_ok()
}

fn pids_listening_on_port(port: u16) -> Result<Vec<i32>, String> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!("tcp:{}", port)])
            .output()
            .map_err(|e| format!("无法检测端口占用: {}", e))?;

        let pids: Vec<i32> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect();
        Ok(pids)
    }
    #[cfg(windows)]
    {
        let needle = format!(":{}", port);
        let output = Command::new("netstat")
            .args(["-ano", "-p", "tcp"])
            .output()
            .map_err(|e| format!("无法检测端口占用: {}", e))?;

        let mut pids = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if line.contains("LISTENING") && line.contains(&needle) {
                if let Some(pid) = line.split_whitespace().last().and_then(|s| s.parse().ok()) {
                    pids.push(pid);
                }
            }
        }
        pids.sort_unstable();
        pids.dedup();
        Ok(pids)
    }
}

fn signal_pids(pids: &[i32], force: bool) {
    for pid in pids {
        #[cfg(unix)]
        unsafe {
            let sig = if force { libc::SIGKILL } else { libc::SIGTERM };
            libc::kill(*pid, sig);
        }
        #[cfg(windows)]
        {
            let mut cmd = Command::new("taskkill");
            cmd.args(["/PID", &pid.to_string()]);
            if force {
                cmd.arg("/F");
            }
            let _ = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
    }
}

/// Terminate processes listening on `port` (SIGTERM → wait → SIGKILL).
fn force_release_port(port: u16) -> Result<(), String> {
    if !is_port_listening(port) {
        return Ok(());
    }

    let pids = pids_listening_on_port(port)?;
    if !pids.is_empty() {
        signal_pids(&pids, false);
        if wait_port_free(port, Duration::from_secs(3)) {
            return Ok(());
        }
        signal_pids(&pids, true);
        if wait_port_free(port, Duration::from_secs(2)) {
            return Ok(());
        }
    } else if wait_port_free(port, Duration::from_secs(2)) {
        return Ok(());
    }

    Err(format!("无法释放端口 {}，请手动关闭占用程序", port))
}

/// Reclaim a port left by a previous Void proxy instance (health check must pass).
pub fn reclaim_port(port: u16) -> Result<(), String> {
    if port < 1024 {
        return Err("端口必须在 1024-65535 之间".to_string());
    }
    if !is_port_listening(port) {
        return Ok(());
    }
    if !is_likely_our_proxy(port) {
        return Err(format!("端口 {} 被其他程序占用，无法自动释放", port));
    }
    force_release_port(port)
}

fn find_next_free_port(from: u16) -> Option<u16> {
    scan_next_free_port(from, |p| is_port_listening(p), PORT_SCAN_MAX)
}

/// Pure scan helper — testable without binding sockets.
pub(crate) fn scan_next_free_port(
    from: u16,
    is_occupied: impl Fn(u16) -> bool,
    max_scan: u16,
) -> Option<u16> {
    for offset in 1..=max_scan {
        let candidate = from as u32 + u32::from(offset);
        if candidate > 65535 {
            break;
        }
        let candidate = candidate as u16;
        if !is_occupied(candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Prefer `preferred`. Reclaim if occupied by our stale proxy; fallback only when allowed.
fn prepare_listen_port(preferred: u16, allow_fallback: bool) -> Result<(u16, bool, bool), String> {
    if !is_port_listening(preferred) {
        return Ok((preferred, false, false));
    }

    if is_likely_our_proxy(preferred) {
        force_release_port(preferred)?;
        return Ok((preferred, false, true));
    }

    if allow_fallback {
        if let Some(next) = find_next_free_port(preferred) {
            return Ok((next, true, false));
        }
        return Err(format!(
            "端口 {} 被其他程序占用，且 {}–{} 无可用端口",
            preferred,
            preferred + 1,
            preferred.saturating_add(PORT_SCAN_MAX).min(65535)
        ));
    }

    Err(format!(
        "端口 {} 已被其他程序占用。请先关闭占用程序，或勾选「占用时自动换端口」",
        preferred
    ))
}

fn http_post_json(port: u16, path: &str) -> Result<Value, String> {
    let mut stream = TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_secs(5),
    )
    .map_err(|_| "无法连接本地服务".to_string())?;

    let req = format!(
        "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        path, port
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("请求失败: {}", e))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let resp = String::from_utf8_lossy(&buf);
    let body = resp
        .split("\r\n\r\n")
        .nth(1)
        .or_else(|| resp.split("\n\n").nth(1))
        .unwrap_or("")
        .trim();

    if !resp.contains("200 OK") {
        return Err(format!(
            "HTTP 错误: {}",
            resp.lines().next().unwrap_or("unknown")
        ));
    }

    serde_json::from_str(body).map_err(|e| format!("解析响应失败: {}", e))
}

fn running_status(state: &ClashServiceState) -> ServiceStatus {
    let port = state.active_port.lock().ok().and_then(|p| *p);
    let url = state.active_url.lock().ok().and_then(|u| u.clone());
    ServiceStatus {
        status: "running".to_string(),
        port,
        url,
        base_url: port.map(|p| format!("http://127.0.0.1:{}", p)),
        runner_kind: state.runner.label().to_string(),
    }
}

fn stopped_status(state: &ClashServiceState) -> ServiceStatus {
    ServiceStatus {
        status: "stopped".to_string(),
        port: None,
        url: None,
        base_url: None,
        runner_kind: state.runner.label().to_string(),
    }
}

fn clear_running_state(state: &ClashServiceState, child: &mut Option<Child>) {
    if let Some(mut c) = child.take() {
        kill_child(&mut c);
    }
    if let Ok(mut p) = state.active_port.lock() {
        *p = None;
    }
    if let Ok(mut u) = state.active_url.lock() {
        *u = None;
    }
}

// ─── Public API ────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct StartServiceResult {
    pub base_url: String,
    pub port: u16,
    pub requested_port: u16,
    pub port_changed: bool,
    pub port_reclaimed: bool,
}

#[derive(Serialize, Clone)]
pub struct PortCheckResult {
    pub available: bool,
    pub port: u16,
    pub reclaimable: bool,
    pub foreign: bool,
    pub suggested_port: Option<u16>,
}

pub fn check_port(port: u16) -> Result<PortCheckResult, String> {
    if port < 1024 {
        return Err("端口必须在 1024-65535 之间".to_string());
    }
    if !is_port_listening(port) {
        return Ok(PortCheckResult {
            available: true,
            port,
            reclaimable: false,
            foreign: false,
            suggested_port: None,
        });
    }
    if is_likely_our_proxy(port) {
        return Ok(PortCheckResult {
            available: false,
            port,
            reclaimable: true,
            foreign: false,
            suggested_port: None,
        });
    }
    Ok(PortCheckResult {
        available: false,
        port,
        reclaimable: false,
        foreign: true,
        suggested_port: find_next_free_port(port),
    })
}

pub fn start_service(
    state: &ClashServiceState,
    url: String,
    port: u16,
    allow_fallback: bool,
) -> Result<StartServiceResult, String> {
    let trimmed = validate_subscription_url(&url)?;

    if port < 1024 {
        return Err("端口必须在 1024-65535 之间".to_string());
    }

    let mut child_lock = state.child.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut child) = *child_lock {
        match child.try_wait() {
            Ok(Some(_)) => *child_lock = None,
            Ok(None) => {
                let active = state.active_port.lock().ok().and_then(|p| *p);
                return Err(format!("服务已在运行 (port {})", active.unwrap_or(port)));
            }
            Err(_) => *child_lock = None,
        }
    }

    let (actual_port, port_changed, port_reclaimed) = prepare_listen_port(port, allow_fallback)?;

    let stderr_path = stderr_temp_path();
    let _ = std::fs::remove_file(&stderr_path);

    let mut child = sidecar::spawn_proxy(&state.runner, actual_port, &trimmed).map_err(|e| {
        match &state.runner {
            ProxyRunner::Sidecar(_) => format!("启动内置代理服务失败: {}", e),
            ProxyRunner::NodeScript { .. } => format!(
                "启动失败: {}（请安装 Node.js 18+，或运行 pnpm build:sidecar 构建内置服务）",
                e
            ),
        }
    })?;

    if let Some(stderr) = child.stderr.take() {
        let path = stderr_path.clone();
        std::thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stderr);
            let mut output = String::new();
            let _ = reader.read_to_string(&mut output);
            let _ = std::fs::write(&path, &output);
        });
    }

    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut health_ok = false;

    while std::time::Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(status)) => {
                std::thread::sleep(Duration::from_millis(100));
                let stderr_detail = std::fs::read_to_string(&stderr_path)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let _ = std::fs::remove_file(&stderr_path);
                let detail = if stderr_detail.is_empty() {
                    String::new()
                } else {
                    format!(": {}", stderr_detail)
                };
                return Err(format!(
                    "服务进程异常退出 (code={:?}){}",
                    status.code(),
                    detail
                ));
            }
            Ok(None) => {}
            Err(e) => return Err(format!("进程检查失败: {}", e)),
        }

        if health_check(actual_port).is_ok() {
            health_ok = true;
            break;
        }

        std::thread::sleep(Duration::from_millis(300));
    }

    if !health_ok {
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_file(&stderr_path);
        return Err(format!("服务启动超时 ({}s)", deadline.elapsed().as_secs()));
    }

    let _ = std::fs::remove_file(&stderr_path);
    *child_lock = Some(child);
    if let Ok(mut port_lock) = state.active_port.lock() {
        *port_lock = Some(actual_port);
    }
    if let Ok(mut url_lock) = state.active_url.lock() {
        *url_lock = Some(trimmed);
    }

    Ok(StartServiceResult {
        base_url: format!("http://127.0.0.1:{}", actual_port),
        port: actual_port,
        requested_port: port,
        port_changed,
        port_reclaimed,
    })
}

pub fn get_service_status(state: &ClashServiceState) -> ServiceStatus {
    let mut child_lock = match state.child.lock() {
        Ok(c) => c,
        Err(_) => return stopped_status(state),
    };

    match child_lock.as_mut() {
        Some(child) => match child.try_wait() {
            Ok(Some(_)) => {
                clear_running_state(state, &mut *child_lock);
                stopped_status(state)
            }
            Ok(None) => {
                let port = state.active_port.lock().ok().and_then(|p| *p);
                if let Some(p) = port {
                    if health_check(p).is_ok() {
                        running_status(state)
                    } else {
                        clear_running_state(state, &mut *child_lock);
                        stopped_status(state)
                    }
                } else {
                    clear_running_state(state, &mut *child_lock);
                    stopped_status(state)
                }
            }
            Err(_) => {
                clear_running_state(state, &mut *child_lock);
                stopped_status(state)
            }
        },
        None => stopped_status(state),
    }
}

pub fn refresh_service(state: &ClashServiceState) -> Result<Value, String> {
    let status = get_service_status(state);
    if status.status != "running" {
        return Err("服务未运行".to_string());
    }
    let port = status.port.ok_or_else(|| "服务未运行".to_string())?;
    http_post_json(port, "/refresh")
}

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let resource_path = app
        .path()
        .resource_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let bundle_path = resource_path.join("scripts/proxy-server.bundle.cjs");
    let script_path = resource_path.join("scripts/proxy-server.mjs");

    let script_path_str = if bundle_path.exists() {
        bundle_path.to_string_lossy().to_string()
    } else if script_path.exists() {
        script_path.to_string_lossy().to_string()
    } else {
        let dev_bundle = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("scripts/proxy-server.bundle.cjs");
        if dev_bundle.exists() {
            dev_bundle.to_string_lossy().to_string()
        } else {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("scripts/proxy-server.mjs")
                .to_string_lossy()
                .to_string()
        }
    };

    let runner = sidecar::resolve_runner(app.handle(), &script_path_str);

    app.manage(ClashServiceState {
        child: Mutex::new(None),
        active_port: Mutex::new(None),
        active_url: Mutex::new(None),
        runner,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_public_https() {
        let url = validate_subscription_url("https://example.com/sub").unwrap();
        assert_eq!(url, "https://example.com/sub");
    }

    #[test]
    fn validate_rejects_localhost() {
        assert!(validate_subscription_url("http://localhost/sub").is_err());
    }

    #[test]
    fn validate_rejects_private_ip() {
        assert!(validate_subscription_url("http://192.168.1.1/sub").is_err());
        assert!(validate_subscription_url("http://10.0.0.1/sub").is_err());
    }

    #[test]
    fn validate_rejects_invalid_scheme() {
        assert!(validate_subscription_url("ftp://example.com/sub").is_err());
    }

    #[test]
    fn scan_next_free_port_skips_occupied() {
        let occupied = |p: u16| matches!(p, 25501 | 25502);
        assert_eq!(scan_next_free_port(25500, occupied, 32), Some(25503));
    }

    #[test]
    fn scan_next_free_port_none_when_exhausted() {
        assert_eq!(scan_next_free_port(25500, |_| true, 3), None);
    }

    #[test]
    fn scan_respects_max_bound() {
        assert_eq!(scan_next_free_port(65535, |_| true, 5), None);
    }
}
