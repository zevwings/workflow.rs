//! Jira 配置上下文实现
//!
//! 实现 `domain::JiraConfigContext` 和 `client::jira::context::JiraConfigContext` traits，
//! 提供配置获取逻辑。

use std::{path::PathBuf, sync::Arc};

use client::{JiraClientError, JiraConfigContext};
use domain::{GlobalConfigRepository, PathService};

/// Jira 配置上下文实现
///
/// 实现 `JiraConfigContext` trait，提供基于配置适配器的配置获取逻辑。
pub struct JiraConfigContextImpl {
    config: Arc<dyn GlobalConfigRepository>,
    path_service: Arc<dyn PathService>,
}

impl JiraConfigContextImpl {
    pub fn new(
        config: Arc<dyn GlobalConfigRepository>,
        path_service: Arc<dyn PathService>,
    ) -> Self {
        Self {
            config,
            path_service,
        }
    }
}

// 实现 domain 层的 JiraConfigContext
impl JiraConfigContext for JiraConfigContextImpl {
    fn get_jira_email(&self) -> String {
        self.config.load().map(|c| c.jira.email.clone()).unwrap_or_default()
    }

    fn get_jira_api_token(&self) -> String {
        self.config.load().map(|c| c.jira.api_token.clone()).unwrap_or_default()
    }

    fn get_jira_service_address(&self) -> String {
        self.config.load().map(|c| c.jira.service_address.clone()).unwrap_or_default()
    }

    fn get_download_dir(&self) -> Result<PathBuf, JiraClientError> {
        self.path_service
            .get_download_dir()
            .map_err(|e| JiraClientError::ConfigError(format!("Failed to get download dir: {}", e)))
    }
}
