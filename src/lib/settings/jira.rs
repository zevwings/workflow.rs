//! Jira 配置相关结构体

use crate::jira::api::user::JiraUserApi;
use crate::mask_sensitive_value;
use crate::prompt::Tabled;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

/// Jira 配置（TOML）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraSettings {
    /// Jira 用户邮箱（用于 API 认证）
    pub email: Option<String>,
    /// Jira API Token
    pub api_token: Option<String>,
    /// Jira 服务地址
    pub service_address: Option<String>,
}

impl JiraSettings {
    /// 检查 JIRA 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.email.is_none() && self.api_token.is_none() && self.service_address.is_none()
    }
}

/// Jira 配置信息
#[derive(Debug, Clone)]
pub struct JiraConfigInfo {
    /// 邮箱
    pub email: String,
    /// 服务地址
    pub service_address: String,
    /// API Token（掩码显示）
    pub api_token: String,
}

/// Jira 验证状态
#[derive(Debug, Clone)]
pub enum JiraVerificationStatus {
    /// 验证成功
    Success { email: String, account_id: String },
    /// 验证失败
    Failed {
        reason: String,
        details: Vec<String>,
    },
}

/// Jira 验证结果
#[derive(Debug, Clone)]
pub struct JiraVerificationResult {
    /// 是否已配置
    pub configured: bool,
    /// 配置信息（如果已配置）
    pub config: Option<JiraConfigInfo>,
    /// 验证结果
    pub verification: Option<JiraVerificationStatus>,
}

impl JiraSettings {
    /// 验证 Jira 配置并返回结果
    ///
    /// 使用 `JiraUserApi::get_current_user_with_auth` 来验证配置，
    /// 该方法封装了 Jira API 调用逻辑。
    pub fn verify(&self) -> Result<JiraVerificationResult> {
        if let (Some(email), Some(api_token), Some(service_address)) =
            (&self.email, &self.api_token, &self.service_address)
        {
            let config = JiraConfigInfo {
                email: email.clone(),
                service_address: service_address.clone(),
                api_token: mask_sensitive_value(api_token),
            };

            let verification =
                match JiraUserApi::get_current_user_with_auth(email, api_token, service_address) {
                    Ok(user) => Some(JiraVerificationStatus::Success {
                        email: email.clone(),
                        account_id: user.account_id,
                    }),
                    Err(e) => {
                        let error_msg = e.to_string();
                        Some(JiraVerificationStatus::Failed {
                            reason: "Failed to verify Jira configuration".to_string(),
                            details: vec![
                                error_msg,
                                "Please check your Jira service address, email, and API token."
                                    .to_string(),
                            ],
                        })
                    }
                };

            Ok(JiraVerificationResult {
                configured: true,
                config: Some(config),
                verification,
            })
        } else {
            Ok(JiraVerificationResult {
                configured: false,
                config: None,
                verification: None,
            })
        }
    }
}

/// JIRA 配置表格行
///
/// 用于在表格中显示 JIRA 配置信息。
pub struct JiraConfigRow {
    pub email: String,
    pub service_address: String,
    pub api_token: String,
}

impl Tabled for JiraConfigRow {
    fn headers() -> Vec<String> {
        vec![
            "Email".to_string(),
            "Service Address".to_string(),
            "API Token".to_string(),
        ]
    }

    fn row(&self) -> Vec<String> {
        vec![
            self.email.clone(),
            self.service_address.clone(),
            self.api_token.clone(),
        ]
    }
}
