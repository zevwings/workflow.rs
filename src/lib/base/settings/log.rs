//! 日志配置相关结构体

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// 日志配置信息
#[derive(Debug, Clone)]
pub struct LogConfigInfo {
    /// 日志输出文件夹名称
    pub output_folder_name: String,
    /// 日志下载基础目录
    pub download_base_dir: Option<String>,
}

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

/// 日志配置（TOML）
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSettings {
    /// 日志输出文件夹名称
    /// 如果为 `None`，使用默认值 `logs`，且不写入配置文件
    #[serde(default = "LogSettings::default_log_folder_option")]
    pub output_folder_name: Option<String>,
    /// 日志下载基础目录
    #[serde(default = "LogSettings::default_download_base_dir_option")]
    pub download_base_dir: Option<String>,
    /// 日志级别（none, error, warn, info, debug）
    pub level: Option<String>,
    /// 是否同时输出 tracing 日志到控制台（stderr）
    /// 如果为 `true`，tracing 日志会同时输出到文件和控制台
    /// 如果配置文件中不存在此字段，默认为 `false`（只输出到文件）
    /// 注意：只有设置为 `true` 时才会写入配置文件，设置为 `false` 时从配置文件中删除
    pub enable_trace_console: Option<bool>,
}

impl LogSettings {
    /// 检查日志配置是否为空（所有字段都是默认值）
    pub fn is_empty(&self) -> bool {
        let default = LogSettings::default();
        self.output_folder_name == default.output_folder_name
            && self.download_base_dir == default.download_base_dir
            && self.level == default.level
            && self.enable_trace_console == default.enable_trace_console
    }

    /// 默认日志文件夹名称
    pub fn default_log_folder() -> String {
        "logs".to_string()
    }

    /// 默认日志文件夹名称（Option 类型，用于序列化）
    pub fn default_log_folder_option() -> Option<String> {
        None // None 表示使用默认值，不写入配置文件
    }

    /// 获取日志文件夹名称（如果为 None，返回默认值）
    pub fn get_output_folder_name(&self) -> String {
        self.output_folder_name.clone().unwrap_or_else(Self::default_log_folder)
    }

    /// 默认下载基础目录路径（Option 类型，用于序列化）
    /// 返回 `None` 表示使用默认值，不写入配置文件
    pub fn default_download_base_dir_option() -> Option<String> {
        None // None 表示使用默认值，不写入配置文件
    }

    /// 获取日志配置信息
    pub fn get_config_info(&self) -> LogConfigInfo {
        LogConfigInfo {
            output_folder_name: self.get_output_folder_name(),
            download_base_dir: self.download_base_dir.clone(),
        }
    }
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            output_folder_name: Self::default_log_folder_option(), // None
            download_base_dir: Self::default_download_base_dir_option(), // None
            level: None,
            enable_trace_console: None,
        }
    }
}
