//! 默认值辅助函数和实现
//!
//! 提供配置结构体的默认值实现和辅助函数

use super::settings::LogSettings;

// ==================== 默认值辅助函数 ====================

/// 默认日志文件夹名称
pub fn default_log_folder() -> String {
    "logs".to_string()
}

/// 默认下载基础目录路径
pub fn default_download_base_dir() -> String {
    std::env::var("HOME")
        .map(|home| format!("{}/Downloads/Workflow", home))
        .unwrap_or_else(|_| "~/Downloads/Workflow".to_string())
}

/// 默认下载基础目录路径（Option 类型）
pub fn default_download_base_dir_option() -> Option<String> {
    Some(default_download_base_dir())
}

/// 默认 LogSettings 实例
pub fn default_log_settings() -> LogSettings {
    LogSettings {
        delete_when_completed: false,
        output_folder_name: default_log_folder(),
        download_base_dir: default_download_base_dir_option(),
    }
}
