//! GitHub Provider 接口
//!
//! 定义 GitHub API 所需的 Git 操作接口，实现依赖倒置原则。
//! GitHub API 模块通过此接口获取 Git 相关信息，而不直接依赖 Git 实现。

use crate::github::GitHubClientError;

/// GitHub Context trait
///
/// 提供 GitHub API 所需的 Git 仓库相关操作，包括获取远程 URL、当前分支、默认分支等。
/// 通过此 trait，GitHub API 模块可以独立于具体的 Git 实现。
pub trait GitHubConfigContext: Send + Sync {
    /// 获取账号名称
    fn get_name(&self) -> Result<String, GitHubClientError>;
    /// 获取账号邮箱
    fn get_email(&self) -> Result<String, GitHubClientError>;
    /// 获取 API Token
    fn get_api_token(&self) -> Result<String, GitHubClientError>;
}
