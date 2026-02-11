//! GitHub 配置上下文实现
//!
//! 实现 `domain::GitHubContext` trait，提供配置获取逻辑。

use std::sync::Arc;

use domain::{GitHubAccount, GitHubContext, GitHubError, GlobalConfigRepository};

/// GitHub 配置上下文实现
///
/// 实现 `GitHubContext` trait，提供基于配置适配器的配置获取逻辑。
pub struct GitHubContextImpl {
    config: Arc<dyn GlobalConfigRepository>,
}

impl GitHubContextImpl {
    pub fn new(config: Arc<dyn GlobalConfigRepository>) -> Self {
        Self { config }
    }

    /// 获取当前 GitHub 账户配置
    fn get_current_account(&self) -> Result<GitHubAccount, GitHubError> {
        let config = self
            .config
            .load()
            .map_err(|e| GitHubError::ConfigError(format!("Failed to load config: {}", e)))?;
        config
            .github
            .get_current_account()
            .cloned()
            .ok_or_else(|| GitHubError::ConfigError("No GitHub account configured".to_string()))
    }
}

impl GitHubContext for GitHubContextImpl {
    fn get_name(&self) -> Result<String, GitHubError> {
        Ok(self.get_current_account()?.name.clone())
    }

    fn get_email(&self) -> Result<String, GitHubError> {
        Ok(self.get_current_account()?.email.clone())
    }

    fn get_api_token(&self) -> Result<String, GitHubError> {
        Ok(self.get_current_account()?.api_token.clone())
    }
}
