//! 路径管理
//!
//! 统一管理所有路径信息，包括：
//! - 配置文件路径（存储在 `~/.workflow/config/` 目录下）
//! - 安装路径（二进制文件和补全脚本的安装路径和名称）
//! - Completion 目录路径
//!
//! ## 使用方式
//!
//! 所有路径 API 都通过模块级函数直接访问：
//! ```rust
//! use toolkit::paths::{config_dir, workflow_config, binary_install_dir};
//! ```

mod base;
mod config;
mod constants;
mod download;
mod error;
mod expand;
mod info;
mod install;

// 重新导出常量
pub use constants::*;

// 重新导出错误类型
pub use error::PathError;

// ==================== 路径工具方法 ====================

/// 展开路径字符串
///
/// 支持的路径格式：
/// - Unix: `~` 和 `~/path` - 展开为用户主目录
/// - Unix: `$VAR` 和 `${VAR}` - 展开环境变量
/// - Windows: `%VAR%` 和 `%VAR%\path` - 展开环境变量
/// - 绝对路径: 直接使用
///
/// # 示例
///
/// ```text
/// // Unix
/// expand("~/Documents/Workflow") -> "/home/user/Documents/Workflow"
/// expand("~") -> "/home/user"
/// expand("$HOME/Documents") -> "/home/user/Documents"
/// expand("${HOME}/Documents") -> "/home/user/Documents"
///
/// // Windows
/// expand("%USERPROFILE%\\Documents\\Workflow") -> "C:\\Users\\User\\Documents\\Workflow"
/// expand("%APPDATA%\\workflow") -> "C:\\Users\\User\\AppData\\Roaming\\workflow"
///
/// // 绝对路径
/// expand("/absolute/path") -> "/absolute/path"
/// expand("C:\\absolute\\path") -> "C:\\absolute\\path"
/// ```
pub use expand::expand;

// ==================== 下载路径相关方法 ====================

pub use download::default_download_base_dir;

// ==================== 配置路径相关方法 ====================

pub use config::{
    commands_config_path, config_dir, jira_config_path, llm_config_path, logs_dir,
    project_config_file, project_config_path, repo_dir, repo_workflow_dir, repository_config_path,
    user_config_file, work_history_dir, workflow_config_path, workflow_dir,
};

// ==================== 安装路径相关方法 ====================

pub use install::{
    binary_install_dir, binary_name, binary_paths, command_names, completion_cache_dir,
    completion_cache_dir_shell_path, completion_dir, completion_dir_shell_path,
    completion_file_shell_path, completion_source_shell_path,
};

// ==================== 信息查询 API ====================

pub use info::{is_config_in_icloud, storage_info, storage_location};
