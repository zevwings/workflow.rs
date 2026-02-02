//! Jira 配置相关结构体

use serde::{Deserialize, Serialize};

/// Jira 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraSettings {
    /// Jira 用户邮箱（用于 API 认证）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub email: String,
    /// Jira API Token
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_token: String,
    /// Jira 服务地址
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub service_address: String,
}

impl JiraSettings {
    /// 检查 JIRA 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.email.is_empty() && self.api_token.is_empty() && self.service_address.is_empty()
    }
}
