use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use rinova_proxy_sdk::{start_server, RuleMode, ServerHandle, ServerOptions};
use serde::Serialize;
use serde_json::Value;
use tauri::Manager;

const REFRESH_INTERVAL_MIN: u64 = 60;
const RUNNER_BUILTIN: &str = "builtin";

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
    pub server: Mutex<Option<ServerHandle>>,
    pub active_port: Mutex<Option<u16>>,
    pub active_url: Mutex<Option<String>>,
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

// ─── Embedded proxy server ─────────────────────────────────

pub async fn stop_service_impl(state: &ClashServiceState) {
    let handle = {
        let mut server_lock = match state.server.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        server_lock.take()
    };

    if let Some(handle) = handle {
        handle.shutdown().await;
    }

    if let Ok(mut port_lock) = state.active_port.lock() {
        *port_lock = None;
    }
    if let Ok(mut url_lock) = state.active_url.lock() {
        *url_lock = None;
    }
}

/// Sync wrapper for app exit / tray quit (off the async command path).
pub fn stop_service_blocking(state: &ClashServiceState) {
    tauri::async_runtime::block_on(stop_service_impl(state));
}

// ─── HTTP helpers ──────────────────────────────────────────

fn configure_probe_timeouts(stream: &TcpStream) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| format!("设置读超时失败: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| format!("设置写超时失败: {e}"))?;
    Ok(())
}

fn health_response_ok(resp: &str) -> bool {
    resp.contains("200 OK")
        || resp.contains("\"status\":\"ok\"")
        || resp.contains("\"status\": \"ok\"")
}

fn health_check(port: u16) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| format!("地址解析失败: {e}"))?,
        Duration::from_secs(2),
    )
    .map_err(|_| format!("端口 {port} 未监听到服务"))?;
    configure_probe_timeouts(&stream)?;

    let req = format!(
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|_| "发送请求失败".to_string())?;

    let mut buf = [0u8; 512];
    let n = stream
        .read(&mut buf)
        .map_err(|_| "读取响应失败".to_string())?;
    let resp = String::from_utf8_lossy(&buf[..n]);

    if health_response_ok(&resp) {
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

fn current_pid() -> i32 {
    std::process::id() as i32
}

fn pids_listening_on_port(port: u16) -> Result<Vec<i32>, String> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!("tcp:{port}")])
            .output()
            .map_err(|e| format!("无法检测端口占用: {e}"))?;

        let pids: Vec<i32> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect();
        Ok(pids)
    }
    #[cfg(windows)]
    {
        let needle = format!(":{port}");
        let output = Command::new("netstat")
            .args(["-ano", "-p", "tcp"])
            .output()
            .map_err(|e| format!("无法检测端口占用: {e}"))?;

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

/// Terminate **other** processes listening on `port`. Never kills the current process.
fn force_release_port(port: u16) -> Result<(), String> {
    if !is_port_listening(port) {
        return Ok(());
    }

    let self_pid = current_pid();
    let pids: Vec<i32> = pids_listening_on_port(port)?
        .into_iter()
        .filter(|pid| *pid != self_pid)
        .collect();

    if pids.is_empty() {
        return Err(format!(
            "端口 {port} 正由本应用占用，请先在应用内停止服务"
        ));
    }

    signal_pids(&pids, false);
    if wait_port_free(port, Duration::from_secs(3)) {
        return Ok(());
    }
    signal_pids(&pids, true);
    if wait_port_free(port, Duration::from_secs(2)) {
        return Ok(());
    }

    Err(format!("无法释放端口 {port}，请手动关闭占用程序"))
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
        return Err(format!("端口 {port} 被其他程序占用，无法自动释放"));
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
            "端口 {preferred} 被其他程序占用，且 {}–{} 无可用端口",
            preferred + 1,
            preferred.saturating_add(PORT_SCAN_MAX).min(65535)
        ));
    }

    Err(format!(
        "端口 {preferred} 已被其他程序占用。请先关闭占用程序，或勾选「占用时自动换端口」"
    ))
}

fn http_post_json(port: u16, path: &str) -> Result<Value, String> {
    let mut stream = TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().unwrap(),
        Duration::from_secs(5),
    )
    .map_err(|_| "无法连接本地服务".to_string())?;
    configure_probe_timeouts(&stream)?;

    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("请求失败: {e}"))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .map_err(|e| format!("读取响应失败: {e}"))?;

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

    serde_json::from_str(body).map_err(|e| format!("解析响应失败: {e}"))
}

fn running_status(state: &ClashServiceState) -> ServiceStatus {
    let port = state.active_port.lock().ok().and_then(|p| *p);
    let url = state.active_url.lock().ok().and_then(|u| u.clone());
    ServiceStatus {
        status: "running".to_string(),
        port,
        url,
        base_url: port.map(|p| format!("http://127.0.0.1:{p}")),
        runner_kind: RUNNER_BUILTIN.to_string(),
    }
}

fn stopped_status() -> ServiceStatus {
    ServiceStatus {
        status: "stopped".to_string(),
        port: None,
        url: None,
        base_url: None,
        runner_kind: String::new(),
    }
}

/// Whether the embedded service should be considered running for status polling.
pub(crate) fn service_is_alive(has_handle: bool, port: Option<u16>) -> bool {
    if !has_handle {
        return false;
    }
    port.is_some_and(is_port_listening)
}

/// Wait until the TCP listener accepts connections. Avoids blocking HTTP reads on the async runtime.
async fn wait_for_listen(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if is_port_listening(port) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

fn detach_stale_service(state: &ClashServiceState) {
    let handle = state
        .server
        .lock()
        .ok()
        .and_then(|mut guard| guard.take());

    if let Ok(mut port_lock) = state.active_port.lock() {
        *port_lock = None;
    }
    if let Ok(mut url_lock) = state.active_url.lock() {
        *url_lock = None;
    }

    if let Some(handle) = handle {
        tauri::async_runtime::spawn(async move {
            handle.shutdown().await;
        });
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
        let reclaimable = pids_listening_on_port(port)
            .map(|pids| pids.iter().any(|pid| *pid != current_pid()))
            .unwrap_or(false);
        return Ok(PortCheckResult {
            available: false,
            port,
            reclaimable,
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

pub async fn start_service(
    state: &ClashServiceState,
    url: String,
    port: u16,
    allow_fallback: bool,
) -> Result<StartServiceResult, String> {
    let trimmed = validate_subscription_url(&url)?;

    if port < 1024 {
        return Err("端口必须在 1024-65535 之间".to_string());
    }

    {
        let server_lock = state.server.lock().map_err(|e| e.to_string())?;
        if server_lock.is_some() {
            let active = state.active_port.lock().ok().and_then(|p| *p);
            return Err(format!("服务已在运行 (port {})", active.unwrap_or(port)));
        }
    }

    let (actual_port, port_changed, port_reclaimed) = prepare_listen_port(port, allow_fallback)?;

    let handle = start_server(ServerOptions {
        url: trimmed.clone(),
        port: actual_port,
        interval_min: REFRESH_INTERVAL_MIN,
        rule_mode: RuleMode::Builtin,
    })
    .await
    .map_err(|e| format!("启动内置代理服务失败: {e}"))?;

    if !wait_for_listen(actual_port, Duration::from_secs(3)).await {
        handle.shutdown().await;
        return Err("服务启动超时 (3s)".to_string());
    }

    {
        let mut server_lock = state.server.lock().map_err(|e| e.to_string())?;
        *server_lock = Some(handle);
    }
    if let Ok(mut port_lock) = state.active_port.lock() {
        *port_lock = Some(actual_port);
    }
    if let Ok(mut url_lock) = state.active_url.lock() {
        *url_lock = Some(trimmed);
    }

    Ok(StartServiceResult {
        base_url: format!("http://127.0.0.1:{actual_port}"),
        port: actual_port,
        requested_port: port,
        port_changed,
        port_reclaimed,
    })
}

pub fn get_service_status(state: &ClashServiceState) -> ServiceStatus {
    let has_handle = state
        .server
        .lock()
        .ok()
        .map(|guard| guard.is_some())
        .unwrap_or(false);

    if !has_handle {
        return stopped_status();
    }

    let port = state.active_port.lock().ok().and_then(|p| *p);
    if service_is_alive(has_handle, port) {
        return running_status(state);
    }

    detach_stale_service(state);
    stopped_status()
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
    app.manage(ClashServiceState {
        server: Mutex::new(None),
        active_port: Mutex::new(None),
        active_url: Mutex::new(None),
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

    #[test]
    fn stopped_status_has_no_runner_label() {
        assert!(stopped_status().runner_kind.is_empty());
    }

    #[test]
    fn health_response_ok_matches_status_variants() {
        assert!(health_response_ok("HTTP/1.1 200 OK\r\n\r\n"));
        assert!(health_response_ok(r#"{"status":"ok","nodes":1}"#));
        assert!(health_response_ok(r#"{"status": "ok", "nodes": 1}"#));
        assert!(!health_response_ok(r#"{"status":"initializing"}"#));
    }

    #[test]
    fn service_is_alive_requires_handle_and_listening_port() {
        assert!(!service_is_alive(false, Some(25500)));
        assert!(!service_is_alive(true, None));
        assert!(!service_is_alive(true, Some(1)));
    }
}
