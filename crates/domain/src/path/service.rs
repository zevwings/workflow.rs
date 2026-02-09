//! 路径服务抽象
//!
//! 统一后缀命名：
//! - 目录：`*_dir`（返回 `PathBuf`，或 `*_dir_shell_path` 返回 `String`）
//! - 文件：`*_filepath`（返回 `PathBuf`，或 `*_filepath_shell_path` 返回 `String`）

use std::path::PathBuf;

use crate::path::error::PathError;

/// 路径服务：统一提供工作流相关目录与文件路径
///
/// 实现本 trait 的类型根据运行环境（如是否启用 iCloud）解析并返回配置目录、
/// 安装目录、下载目录、补全脚本目录等路径，供上层业务使用。
///
/// # 命名约定
///
/// - **目录**：`*_dir` 返回 [`PathBuf`]，`*_shell_dir` 返回供 shell 使用的字符串
/// - **文件**：`*_filepath` 返回 [`PathBuf`]，`*_shell_path` 返回供 shell 使用的字符串
///
/// # 线程安全
///
/// 实现须满足 [`Send`] + [`Sync`]，以便在多线程或异步上下文中共享。
pub trait PathService: Send + Sync {
    /// 获取工作流主目录的路径。
    fn get_workflow_config_dir(&self) -> Result<PathBuf, PathError>;

    // ----------- workflow 相关路径（均使用本地目录，需随 iCloud 同步） -----------

    /// 获取工作流主配置文件的路径（如 `config.toml` 所在位置）。
    fn get_workflow_config_filepath(&self) -> Result<PathBuf, PathError>;

    /// 获取 Jira 相关配置文件的路径。
    fn get_jira_config_filepath(&self) -> Result<PathBuf, PathError>;

    /// 获取 Jira 工作历史目录的路径。
    fn get_jira_work_history_dir(&self) -> Result<PathBuf, PathError>;

    // ----------- 仓库级别配置文件路径 -----------

    /// 获取项目配置文件目录的路径。
    fn get_project_config_dir(&self) -> Result<PathBuf, PathError>;

    /// 获取项目配置文件的路径。
    fn get_project_config_filepath(&self) -> Result<PathBuf, PathError>;

    /// 获取用户配置文件的路径。
    fn get_user_config_filepath(&self) -> Result<PathBuf, PathError>;

    /// 获取 MCP 配置文件的路径。
    fn get_mcp_config_filepath(&self) -> Result<PathBuf, PathError>;

    // ----------- binary 相关路径（均使用本地目录，不随 iCloud 同步） -----------

    /// 获取二进制可执行文件的安装根目录（不含可执行文件名本身）。
    fn get_binary_install_dir(&self) -> Result<PathBuf, PathError>;

    /// 获取当前平台下的二进制文件名（如 Unix 下 `workflow`，Windows 下 `workflow.exe`）。
    fn get_binary_name(&self) -> Result<String, PathError>;

    // ----------- download 相关路径（均使用本地目录，不随 iCloud 同步） -----------

    /// 获取下载基础目录；下载内容仅存于本地，不参与 iCloud 同步。
    fn get_download_dir(&self) -> Result<PathBuf, PathError>;

    // ----------- completion 相关路径（均使用本地目录，不随 iCloud 同步） -----------

    /// 获取补全脚本所在目录（强制本地，如 `~/.workflow/completions/`），返回 [`PathBuf`] 供程序内使用。
    fn get_completion_dir(&self) -> Result<PathBuf, PathError>;

    /// 获取补全缓存目录（强制本地，如 `~/.workflow/.completion_cache/`），返回 [`PathBuf`] 供程序内使用。
    fn get_completion_cache_dir(&self) -> Result<PathBuf, PathError>;

    /// 获取补全配置文件路径（如 `~/.workflow/completions/.completions`），返回 [`PathBuf`] 供程序内使用。
    fn get_completion_config_filepath(&self) -> Result<PathBuf, PathError>;

    // ----------- logs 相关路径（均使用本地目录，不随 iCloud 同步） -----------

    /// 获取日志目录路径（强制本地，不同步）。
    fn get_logs_dir(&self) -> Result<PathBuf, PathError>;
}
