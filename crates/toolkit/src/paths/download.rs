//! 下载路径管理

/// 默认下载基础目录路径
///
/// 跨平台支持：
/// - Unix (macOS/Linux): `~/Documents/Workflow`
/// - Windows: `%USERPROFILE%\Documents\Workflow`
pub fn default_download_base_dir() -> String {
    // 使用 dirs::home_dir() 获取主目录
    dirs::home_dir()
        .map(|h| h.join("Documents").join("Workflow").to_string_lossy().to_string())
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                "C:\\Users\\User\\Documents\\Workflow".to_string()
            } else {
                "~/Documents/Workflow".to_string()
            }
        })
}
