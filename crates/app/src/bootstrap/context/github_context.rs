//! GitHub 配置上下文实现
//!
//! 实现 `domain::GitHubContext` 和 `client::github::context::GitHubConfigContext` traits，
//! 提供配置获取逻辑。

use std::sync::Arc;

use client::{GitHubClientError, GitHubConfigContext};
use domain::{GitHubAccount, GlobalConfigRepository};

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
    fn get_current_account(&self) -> Result<GitHubAccount, GitHubClientError> {
        let config = self
            .config
            .load()
            .map_err(|e| GitHubClientError::ConfigError(format!("Failed to load config: {}", e)))?;
        config.github.get_current_account().cloned().ok_or_else(|| {
            GitHubClientError::ConfigError("No GitHub account configured".to_string())
        })
    }
}

// 实现 domain 层的 GitHubContext
impl GitHubConfigContext for GitHubContextImpl {
    fn get_name(&self) -> Result<String, GitHubClientError> {
        Ok(self.get_current_account()?.name.clone())
    }

    fn get_email(&self) -> Result<String, GitHubClientError> {
        Ok(self.get_current_account()?.email.clone())
    }

    fn get_api_token(&self) -> Result<String, GitHubClientError> {
        Ok(self.get_current_account()?.api_token.clone())
    }
}
