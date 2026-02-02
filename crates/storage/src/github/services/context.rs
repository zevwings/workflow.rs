//! GitHub 服务上下文
//!
//! 提供 GitHub 服务共用的上下文和辅助方法

use std::sync::Arc;

use domain::{GitHubError, GitRepoRepository};

pub trait ServiceContext: Send + Sync {
    fn get_owner_and_repo(&self) -> Result<(String, String), GitHubError>;

    /// 解析 PR ID 为 PR number
    fn parse_pr_number(&self, pr_id: &str) -> Result<u64, GitHubError> {
        pr_id.parse::<u64>().map_err(|_| {
            GitHubError::ApiError(
                "Invalid PR number: expected numeric PR ID (e.g., '123')".to_string(),
            )
        })
    }
}

/// GitHub 服务上下文
///
/// 封装服务共用的依赖和辅助方法
pub struct ServiceContextImpl {
    repo_repository: Arc<dyn GitRepoRepository>,
}

impl ServiceContextImpl {
    /// 创建新的服务上下文
    pub fn new(repo_repository: Arc<dyn GitRepoRepository>) -> Self {
        Self { repo_repository }
    }
}

impl ServiceContext for ServiceContextImpl {
    /// 从 repo_repository 获取 owner 和 repo_name
    fn get_owner_and_repo(&self) -> Result<(String, String), GitHubError> {
        let repo_info = self.repo_repository.get_repo_info();

        let owner = repo_info.owner.ok_or_else(|| {
            GitHubError::ApiError("Failed to get repository owner from repo info".to_string())
        })?;

        let repo_name = repo_info
            .name
            .ok_or_else(|| {
                GitHubError::ApiError("Failed to get repository name from repo info".to_string())
            })?
            .split('/')
            .nth(1)
            .ok_or_else(|| {
                GitHubError::ApiError(
                    "Failed to parse repository name (expected owner/repo format)".to_string(),
                )
            })?
            .to_string();

        Ok((owner, repo_name))
    }
}
