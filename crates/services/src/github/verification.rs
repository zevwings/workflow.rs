//! GitHub 认证验证服务实现

use std::sync::Arc;

use client::GitHubClient;
use domain::{GitHubError, GitHubUser, GitHubVerificationService};

/// GitHub 认证验证服务实现
pub struct GitHubVerificationServiceImpl {
    client: Arc<dyn GitHubClient>,
}

impl GitHubVerificationServiceImpl {
    /// 创建新的服务实例
    pub fn new(client: Arc<dyn GitHubClient>) -> Self {
        Self { client }
    }
}

impl GitHubVerificationService for GitHubVerificationServiceImpl {
    fn get_user_info(&self) -> Result<GitHubUser, GitHubError> {
        let url = "/user";
        let response = self.client.get(url)?;
        let json_value = response
            .json()
            .map_err(|e| GitHubError::ApiError(format!("Failed to parse response JSON: {}", e)))?;
        let user: GitHubUser = serde_json::from_value(json_value).map_err(|e| {
            GitHubError::ApiError(format!("Failed to deserialize user info: {}", e))
        })?;

        Ok(user)
    }
}
