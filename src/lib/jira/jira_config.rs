//! Jira 配置提供者接口
//!
//! 定义 Jira 配置的抽象接口，实现依赖倒置原则。
//! Jira 模块定义此接口，由其他模块（如 infra）实现。

use color_eyre::Result;
use std::path::PathBuf;

/// Jira 配置提供者 trait
///
/// 提供 Jira 相关的配置信息，包括认证信息、服务地址和路径配置。
/// 通过此 trait，Jira 模块可以独立于具体的配置实现（如 Settings）。
///
/// # 实现者
///
/// 此 trait 应该由基础设施层（如 `infra::adapters`）实现，将不同的配置源
/// （如 Settings、环境变量等）适配为统一的接口。
///
/// # 线程安全
///
/// 此 trait 要求实现 `Send + Sync`，以便在多线程环境中安全使用。
pub trait JiraConfigProvider: Send + Sync {
    /// 获取 Jira 用户邮箱
    ///
    /// # 返回
    ///
    /// 返回 Jira 用户邮箱，如果未配置则返回 `None`。
    fn get_jira_email(&self) -> Option<String>;

    /// 获取 Jira API Token
    ///
    /// # 返回
    ///
    /// 返回 Jira API Token，如果未配置则返回 `None`。
    fn get_jira_api_token(&self) -> Option<String>;

    /// 获取 Jira 服务地址
    ///
    /// # 返回
    ///
    /// 返回 Jira 服务地址，如果未配置则返回 `None`。
    fn get_jira_service_address(&self) -> Option<String>;

    /// 获取下载基础目录
    ///
    /// # 返回
    ///
    /// 返回下载基础目录路径（已展开），如果未配置则返回默认值。
    fn get_download_base_dir(&self) -> Result<PathBuf>;

    /// 获取日志输出文件夹名称
    ///
    /// # 返回
    ///
    /// 返回日志输出文件夹名称，如果未配置则返回默认值。
    fn get_log_output_folder_name(&self) -> String;

    /// 获取 Jira 配置文件路径
    ///
    /// # 返回
    ///
    /// 返回 Jira 配置文件的路径（`~/.workflow/config/jira.toml`）。
    fn get_jira_config_path(&self) -> Result<PathBuf>;

    /// 获取工作历史目录路径
    ///
    /// # 返回
    ///
    /// 返回工作历史目录的路径（`~/.workflow/work-history/`）。
    fn get_work_history_dir(&self) -> Result<PathBuf>;
}
