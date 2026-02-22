//! GitHub 认证验证接口

use crate::github::{entity::GitHubUser, error::GitHubError};

/// GitHub 认证验证接口
///
/// 仅用于验证 GitHub token 是否有效，不依赖任何 Git 仓库状态。
pub trait GitHubVerificationService: Send + Sync {
    /// 获取用户信息
    ///
    /// 通过调用 GitHub API 的 `/user` 端点来验证 token 并获取用户信息。
    /// 如果成功，返回用户信息；如果失败（如 token 无效、网络问题），返回错误。
    fn get_user_info(&self) -> Result<GitHubUser, GitHubError>;
}
