//! 仓库配置仓储接口
//!
//! 提供仓库配置（ProjectConfig、UserConfig 和 RepoConfig）的持久化操作接口。

use crate::{
    config::repo::{ProjectConfig, RepoConfig, UserConfig},
    errors::ServiceError,
};

/// 仓库配置仓储接口
///
/// 负责管理仓库配置的加载和保存操作。
/// 这是存储层的接口，提供底层的文件读写能力。
///
/// 因此不需要通过参数传入。
pub trait RepoConfigRepository: Send + Sync {
    /// 加载项目配置
    ///
    /// 从当前工作目录的 `.workflow/config.toml` 加载项目配置。
    ///
    /// # 返回
    ///
    /// 返回 `ProjectConfig`，如果文件不存在则返回默认值。
    fn load_project_config(&self) -> Result<ProjectConfig, ServiceError>;

    /// 保存项目配置
    ///
    /// 保存项目配置到当前工作目录的 `.workflow/config.toml`。
    ///
    /// # 参数
    ///
    /// * `config` - 项目配置
    fn save_project_config(&self, config: &ProjectConfig) -> Result<(), ServiceError>;

    /// 加载用户配置
    ///
    /// 从当前工作目录的 `.workflow/user.toml` 加载仓库级别的用户配置。
    ///
    /// # 返回
    ///
    /// 返回 `UserConfig`，如果文件不存在则返回默认值。
    fn load_user_config(&self) -> Result<UserConfig, ServiceError>;

    /// 保存用户配置
    ///
    /// 保存用户配置到当前工作目录的 `.workflow/user.toml`。
    /// 如果配置为空，则删除文件（如果存在）。
    ///
    /// # 参数
    ///
    /// * `config` - 用户配置
    fn save_user_config(&self, config: &UserConfig) -> Result<(), ServiceError>;

    /// 加载仓库配置
    ///
    /// 从当前工作目录加载项目配置，从用户配置目录加载用户配置，
    /// 以及从 `.cursor/mcp.json` 加载 MCP 配置。
    fn load(&self) -> Result<RepoConfig, ServiceError>;

    /// 保存仓库配置
    ///
    /// 保存项目配置到 `.workflow/config.toml`，保存用户配置到 `~/.workflow/config/repository.toml`，
    /// 保存 MCP 配置到 `.cursor/mcp.json`。
    fn save(&self, config: &RepoConfig) -> Result<(), ServiceError>;
}
