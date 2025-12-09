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
///
/// 跨平台支持：
/// - Unix (macOS/Linux): `~/Documents/Workflow`
/// - Windows: `%USERPROFILE%\Documents\Workflow`
pub fn default_download_base_dir() -> String {
    // 使用 dirs::home_dir() 获取主目录
    dirs::home_dir()
        .map(|h| {
            h.join("Documents")
                .join("Workflow")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                "C:\\Users\\User\\Documents\\Workflow".to_string()
            } else {
                "~/Documents/Workflow".to_string()
            }
        })
}

/// 默认下载基础目录路径（Option 类型）
pub fn default_download_base_dir_option() -> Option<String> {
    Some(default_download_base_dir())
}

/// 默认 LogSettings 实例
pub fn default_log_settings() -> LogSettings {
    LogSettings {
        output_folder_name: default_log_folder(),
        download_base_dir: default_download_base_dir_option(),
        level: None,
        enable_trace_console: None,
    }
}

/// 默认 LLM Provider
pub fn default_llm_provider() -> String {
    "openai".to_string()
}

/// 根据 Provider 获取默认模型
pub fn default_llm_model(provider: &str) -> String {
    match provider {
        "openai" => "gpt-4.0".to_string(),
        "deepseek" => "deepseek-chat".to_string(),
        _ => String::new(), // proxy 必须输入，没有默认值
    }
}

/// 默认 LLM 输出语言
pub fn default_language() -> String {
    "en".to_string()
}
