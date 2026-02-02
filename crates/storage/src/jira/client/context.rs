//! Jira 配置上下文实现
//!
//! 实现 `domain::JiraConfigContext` trait，提供配置获取逻辑。

use std::sync::Arc;

use domain::{jira::context::JiraConfigContext, GlobalConfigRepository};

/// Jira 配置上下文实现
///
/// 实现 `JiraConfigContext` trait，提供基于配置适配器的配置获取逻辑。
pub struct JiraConfigContextImpl {
    config: Arc<dyn GlobalConfigRepository>,
}

impl JiraConfigContextImpl {
    pub fn new(config: Arc<dyn GlobalConfigRepository>) -> Self {
        Self { config }
    }
}

impl JiraConfigContext for JiraConfigContextImpl {
    fn get_jira_email(&self) -> String {
        self.config
            .load()
            .map(|c| c.jira.email.clone())
            .unwrap_or_default()
    }

    fn get_jira_api_token(&self) -> String {
        self.config
            .load()
            .map(|c| c.jira.api_token.clone())
            .unwrap_or_default()
    }

    fn get_jira_service_address(&self) -> String {
        self.config
            .load()
            .map(|c| c.jira.service_address.clone())
            .unwrap_or_default()
    }
}
