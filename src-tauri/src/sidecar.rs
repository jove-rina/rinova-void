use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::clash::resolve_node_path;

/// Resolve bundled proxy-server sidecar (no system Node required).
pub fn resolve_sidecar_path(app: &AppHandle) -> Option<PathBuf> {
    let name = sidecar_filename();

    // Production / tauri dev: next to main executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let path = dir.join(&name);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    // Dev fallback: src-tauri/binaries/
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(&name);
    if dev_path.is_file() {
        return Some(dev_path);
    }

    // Bundled resource parent (some bundle layouts)
    if let Ok(resource_dir) = app.path().resource_dir() {
        if let Some(parent) = resource_dir.parent() {
            let path = parent.join(&name);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    None
}

fn sidecar_filename() -> String {
    #[cfg(windows)]
    {
        "proxy-server.exe".to_string()
    }
    #[cfg(not(windows))]
    {
        "proxy-server".to_string()
    }
}

pub enum ProxyRunner {
    Sidecar(PathBuf),
    NodeScript { node: String, script: String },
}

impl ProxyRunner {
    pub fn label(&self) -> &'static str {
        match self {
            ProxyRunner::Sidecar(_) => "sidecar",
            ProxyRunner::NodeScript { .. } => "node",
        }
    }
}

pub fn resolve_runner(app: &AppHandle, script_path: &str) -> ProxyRunner {
    if let Some(sidecar) = resolve_sidecar_path(app) {
        log::info!("Clash proxy: using bundled sidecar ({})", sidecar.display());
        return ProxyRunner::Sidecar(sidecar);
    }

    log::info!("Clash proxy: sidecar not found, falling back to system Node");
    ProxyRunner::NodeScript {
        node: resolve_node_path(),
        script: script_path.to_string(),
    }
}

pub fn spawn_proxy(
    runner: &ProxyRunner,
    port: u16,
    url: &str,
) -> std::io::Result<std::process::Child> {
    use std::process::{Command, Stdio};

    match runner {
        ProxyRunner::Sidecar(bin) => Command::new(bin)
            .args([port.to_string(), url.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn(),
        ProxyRunner::NodeScript { node, script } => Command::new(node)
            .args([script.as_str(), &port.to_string(), url])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn(),
    }
}

#[cfg(test)]
mod tests {
    use super::sidecar_filename;

    #[test]
    fn sidecar_filename_matches_platform() {
        let name = sidecar_filename();
        #[cfg(windows)]
        assert!(name.ends_with(".exe"));
        #[cfg(not(windows))]
        assert_eq!(name, "proxy-server");
    }
}
