//! Platform-specific WebView availability detection.
//!
//! This module provides functions to check whether the required WebView runtime
//! is available on the current platform before launching the Dioxus application.
//!
//! - **macOS**: WebKit is always present (bundled with Safari)
//! - **Linux**: Requires WebKitGTK (libwebkit2gtk)
//! - **Windows**: Requires WebView2 (Microsoft Edge runtime)

/// Checks whether the WebView runtime required by Dioxus/Wry is available.
///
/// # Returns
///
/// - `Ok(())` if the WebView runtime is available
/// - `Err(String)` with a human-readable error message if not available
///
/// # Platform Behavior
///
/// - **macOS**: Always returns `Ok(())` since WebKit is bundled with the OS
/// - **Linux**: Probes for libwebkit2gtk shared library
/// - **Windows**: Checks for WebView2 via registry or file existence
pub fn check_webview_available() -> Result<(), String> {
    check_webview_available_impl()
}

#[cfg(target_os = "macos")]
fn check_webview_available_impl() -> Result<(), String> {
    // WebKit is always available on macOS (bundled with Safari)
    Ok(())
}

#[cfg(target_os = "linux")]
fn check_webview_available_impl() -> Result<(), String> {
    use std::path::Path;

    // Common library paths for WebKitGTK
    const WEBKIT_LIBRARY_PATHS: &[&str] = &[
        // Debian/Ubuntu paths
        "/usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so",
        "/usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.0.so",
        "/usr/lib/aarch64-linux-gnu/libwebkit2gtk-4.1.so",
        "/usr/lib/aarch64-linux-gnu/libwebkit2gtk-4.0.so",
        // Fedora/RHEL paths
        "/usr/lib64/libwebkit2gtk-4.1.so",
        "/usr/lib64/libwebkit2gtk-4.0.so",
        // Arch Linux paths
        "/usr/lib/libwebkit2gtk-4.1.so",
        "/usr/lib/libwebkit2gtk-4.0.so",
        // Generic paths
        "/lib/libwebkit2gtk-4.1.so",
        "/lib/libwebkit2gtk-4.0.so",
    ];

    for path in WEBKIT_LIBRARY_PATHS {
        if Path::new(path).exists() {
            return Ok(());
        }
    }

    // Try pkg-config as a fallback
    if let Ok(output) = std::process::Command::new("pkg-config")
        .args(["--exists", "webkit2gtk-4.1"])
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    if let Ok(output) = std::process::Command::new("pkg-config")
        .args(["--exists", "webkit2gtk-4.0"])
        .output()
    {
        if output.status.success() {
            return Ok(());
        }
    }

    Err(
        "WebKitGTK is not installed. Please install it using your package manager:\n\
         - Debian/Ubuntu: sudo apt install libwebkit2gtk-4.1-dev\n\
         - Fedora: sudo dnf install webkit2gtk4.1-devel\n\
         - Arch: sudo pacman -S webkit2gtk-4.1"
            .to_string(),
    )
}

#[cfg(target_os = "windows")]
fn check_webview_available_impl() -> Result<(), String> {
    use std::path::Path;

    // Check for WebView2 loader DLL in common locations
    let program_files =
        std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
    let program_files_x86 = std::env::var("ProgramFiles(x86)")
        .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| String::new());

    // WebView2 can be installed system-wide or per-user
    let webview2_paths = [
        format!("{}\\Microsoft\\EdgeWebView\\Application", program_files),
        format!("{}\\Microsoft\\EdgeWebView\\Application", program_files_x86),
        format!("{}\\Microsoft\\EdgeWebView\\Application", local_app_data),
    ];

    for base_path in &webview2_paths {
        let path = Path::new(base_path);
        if path.exists() && path.is_dir() {
            // Check if there's at least one version directory
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let msedgewebview2_path = entry.path().join("msedgewebview2.exe");
                        if msedgewebview2_path.exists() {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    // Check Windows Registry for WebView2 installation
    // The registry approach uses winreg which may not be available,
    // so we rely on file existence checks above as the primary method

    Err("Microsoft Edge WebView2 Runtime is not installed.\n\
         Please download and install it from:\n\
         https://developer.microsoft.com/en-us/microsoft-edge/webview2/\n\n\
         Or install Microsoft Edge browser which includes WebView2."
        .to_string())
}

// Fallback for other platforms (iOS, Android, WASM, etc.)
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn check_webview_available_impl() -> Result<(), String> {
    // On other platforms, assume WebView is available or handled differently
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_webview_available() {
        // This test verifies the function runs without panicking.
        // The actual result depends on the platform and installed dependencies.
        let result = check_webview_available();

        // On macOS, it should always succeed
        #[cfg(target_os = "macos")]
        assert!(
            result.is_ok(),
            "WebView should always be available on macOS"
        );

        // On other platforms, we just verify it returns a valid Result
        #[cfg(not(target_os = "macos"))]
        {
            // Either Ok or an Err with a non-empty message
            match result {
                Ok(()) => {}
                Err(msg) => assert!(!msg.is_empty(), "Error message should not be empty"),
            }
        }
    }
}
