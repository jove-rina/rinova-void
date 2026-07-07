//! macOS 屏幕录制 TCC 预检（Sequoia 对未签名 debug 二进制尤其严格）。

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGRequestScreenCaptureAccess() -> bool;
}

/// 当前进程是否已被 TCC 授予屏幕录制权限。
pub fn has_screen_capture_access() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

/// 触发系统授权弹窗；若仍无权限则返回可读说明。
pub fn ensure_screen_capture_access() -> Result<(), String> {
    unsafe {
        if CGPreflightScreenCaptureAccess() {
            return Ok(());
        }
        CGRequestScreenCaptureAccess();
        if CGPreflightScreenCaptureAccess() {
            return Ok(());
        }
    }
    Err(permission_denied_message())
}

pub fn permission_denied_message() -> String {
    "需要屏幕录制权限。请在「系统设置 → 隐私与安全性 → 屏幕录制」中允许 Void，完全退出后重开。".to_string()
}

pub fn capture_api_failed_message() -> String {
    if !has_screen_capture_access() {
        return permission_denied_message();
    }

    "截屏失败，请重试；若仍失败，可关闭「截屏时隐藏应用」后再试。".to_string()
}
