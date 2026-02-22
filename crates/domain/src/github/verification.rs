//! GitHub 认证验证接口

use crate::github::{entity::GitHubUser, error::GitHubError};

/// 验证 GitHub token 是否有效，不依赖任何 Git 仓库状态。
pub trait GitHubVerificationService: Send + Sync {
    /// 通过 `/user` 端点验证 token 并获取用户信息。
    fn get_user_info(&self) -> Result<GitHubUser, GitHubError>;
}
