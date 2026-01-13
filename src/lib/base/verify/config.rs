//! 配置验证模块
//!
//! 验证各种配置项（Log、LLM、Jira、GitHub）的有效性。

use crate::base::settings::settings::{
    GitHubSettings, JiraSettings, LLMSettings, LogSettings, Settings,
};
use crate::base::settings::settings::{
    GitHubVerificationResult, JiraVerificationResult, LLMConfigInfo, LogConfigInfo,
    VerificationResult,
};
use crate::mask_sensitive_value;
use crate::pr::GitHub;
use color_eyre::Result;
use serde_json::Value;

use crate::base::http::{Authorization, HttpClient, RequestConfig};
use crate::jira::types::JiraUser;

/// 配置验证器
pub struct ConfigVerifier;

impl ConfigVerifier {
    /// 验证所有配置（类似 Go 版本的 verify.Verify*Config）
    ///
    /// 按顺序验证：
    /// - Log 配置
    /// - LLM 配置
    /// - Jira 配置
    /// - GitHub 配置
    pub fn verify_all(settings: &Settings) -> Result<VerificationResult> {
        Ok(VerificationResult {
            log: Self::verify_log_config(&settings.log),
            llm: Self::verify_llm_config(&settings.llm),
            jira: Self::verify_jira_config(&settings.jira)?,
            github: Self::verify_github_config(&settings.github)?,
        })
    }

    /// 验证日志配置
    pub fn verify_log_config(log: &LogSettings) -> LogConfigInfo {
        LogConfigInfo {
            output_folder_name: log.get_output_folder_name(),
            download_base_dir: log.download_base_dir.clone(),
        }
    }

    /// 验证 LLM 配置
    pub fn verify_llm_config(llm: &LLMSettings) -> LLMConfigInfo {
        let current = llm.current_provider();

        // 获取 model（如果有保存的值，否则显示默认值）
        let model = if let Some(ref model) = current.model {
            model.clone()
        } else {
            LLMSettings::default_model(&llm.provider)
        };

        // 组合 model 和 URL（仅在 provider 为 "proxy" 时显示 URL）
        let model_display = if llm.provider == "proxy" {
            if let Some(ref url) = current.url {
                if !url.is_empty() {
                    format!("{}({})", model, url)
                } else {
                    model
                }
            } else {
                model
            }
        } else {
            model
        };

        // 获取 Key（掩码显示）
        let key = current
            .key
            .as_ref()
            .map(|k| mask_sensitive_value(k))
            .unwrap_or_else(|| "-".to_string());

        // 获取 Language（如果有保存的值，否则显示默认值）
        let language = if !llm.language.is_empty() {
            llm.language.clone()
        } else {
            LLMSettings::default_language()
        };

        LLMConfigInfo {
            provider: llm.provider.clone(),
            model: model_display,
            key,
            language,
        }
    }

    /// 验证 Jira 配置
    pub fn verify_jira_config(jira: &JiraSettings) -> Result<JiraVerificationResult> {
        if let (Some(email), Some(api_token), Some(service_address)) =
            (&jira.email, &jira.api_token, &jira.service_address)
        {
            use crate::base::settings::settings::{JiraConfigInfo, JiraVerificationStatus};

            let config = JiraConfigInfo {
                email: email.clone(),
                service_address: service_address.clone(),
                api_token: mask_sensitive_value(api_token),
            };

            let base_url = format!("{}/rest/api/2", service_address);
            let url = format!("{}/myself", base_url);

            let verification = match HttpClient::global() {
                Ok(client) => {
                    let auth = Authorization::new(email, api_token);
                    let config = RequestConfig::<Value, Value>::new().auth(&auth);
                    match client.get(&url, config) {
                        Ok(response) => {
                            // 使用 ensure_success 统一处理成功/失败检查
                            match response.ensure_success() {
                                Ok(success_response) => {
                                    match success_response.as_json::<JiraUser>() {
                                        Ok(user) => Some(JiraVerificationStatus::Success {
                                            email: email.clone(),
                                            account_id: user.account_id,
                                        }),
                                        Err(e) => Some(JiraVerificationStatus::Failed {
                                            reason: "Failed to parse Jira user response"
                                                .to_string(),
                                            details: vec![format!("Error: {}", e)],
                                        }),
                                    }
                                }
                                Err(e) => Some(JiraVerificationStatus::Failed {
                                    reason: "Failed to verify Jira configuration".to_string(),
                                    details: vec![
                                        format!("Error: {}", e),
                                        "Please check your Jira service address, email, and API token."
                                            .to_string(),
                                    ],
                                }),
                            }
                        }
                        Err(e) => Some(JiraVerificationStatus::Failed {
                            reason: "Failed to verify Jira configuration".to_string(),
                            details: vec![
                                format!("Error: {}", e),
                                "Please check your Jira service address, email, and API token."
                                    .to_string(),
                            ],
                        }),
                    }
                }
                Err(e) => Some(JiraVerificationStatus::Failed {
                    reason: "Failed to create HTTP client".to_string(),
                    details: vec![format!("Error: {}", e)],
                }),
            };

            Ok(JiraVerificationResult {
                configured: true,
                config: Some(config),
                verification,
            })
        } else {
            use crate::base::settings::settings::JiraVerificationResult;
            Ok(JiraVerificationResult {
                configured: false,
                config: None,
                verification: None,
            })
        }
    }

    /// 验证 GitHub 配置
    pub fn verify_github_config(github: &GitHubSettings) -> Result<GitHubVerificationResult> {
        use crate::base::settings::settings::{
            GitHubAccountInfo, GitHubVerificationResult, GitHubVerificationSummary,
        };

        if github.accounts.is_empty() {
            return Ok(GitHubVerificationResult {
                configured: false,
                accounts: Vec::new(),
                summary: GitHubVerificationSummary {
                    total_count: 0,
                    success_count: 0,
                    failed_accounts: Vec::new(),
                },
            });
        }

        let mut success_count = 0;
        let mut failed_accounts = Vec::new();
        let mut account_infos = Vec::new();

        for account in &github.accounts {
            let is_current =
                github.current.as_ref().map(|c| c == &account.name).unwrap_or_else(|| {
                    // 如果没有设置 current，第一个账号是当前账号
                    github.accounts.first().map(|a| &a.name) == Some(&account.name)
                });

            // 使用该账号的 token 验证
            let (verification_status, verification_error) =
                match GitHub::get_user_info(Some(&account.api_token)) {
                    Ok(_user) => {
                        success_count += 1;
                        ("Success".to_string(), None)
                    }
                    Err(e) => {
                        failed_accounts.push(account.name.clone());
                        ("Failed".to_string(), Some(format!("{}", e)))
                    }
                };

            account_infos.push(GitHubAccountInfo {
                name: account.name.clone(),
                is_current,
                email: account.email.clone(),
                token: mask_sensitive_value(&account.api_token),
                verification_status,
                verification_error,
            });
        }

        let total_count = github.accounts.len();
        Ok(GitHubVerificationResult {
            configured: true,
            accounts: account_infos,
            summary: GitHubVerificationSummary {
                total_count,
                success_count,
                failed_accounts,
            },
        })
    }
}
