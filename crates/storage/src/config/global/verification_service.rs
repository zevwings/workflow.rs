//! 验证服务实现
//!
//! 实现 VerificationService trait，提供配置验证功能。
//!
//! ## 使用方式
//!
//! ```rust,no_run
//! use domain::VerificationService;
//! use storage::config::global::VerificationServiceImpl;
//! use app::registry;
//!
//! let service = registry::get_verification_service();
//! let result = service.verify_jira_config()?;
//! ```
//!
//! 注意：VerificationDisplayService 仍在 services crate 中。

use std::sync::Arc;

use domain::{
    GitHubAccountInfo, GitHubRepository, GitHubVerificationResult, GitHubVerificationSummary,
    GlobalConfigRepository, JiraConfigInfo, JiraRepository, JiraVerificationResult,
    JiraVerificationStatus, LLMConfig, LLMRepository, LLMSettings, LLMVerificationResult,
    LLMVerificationStatus, LogConfigInfo, LogVerificationResult, ServiceError, VerificationService,
};
use toolkit::Sensitive;

/// 验证服务实现
pub struct VerificationServiceImpl {
    config_repository: Arc<dyn GlobalConfigRepository>,
    llm_repository: Arc<dyn LLMRepository>,
    jira_repository: Arc<dyn JiraRepository>,
    github_repository: Arc<dyn GitHubRepository>,
}

impl VerificationServiceImpl {
    pub fn new(
        config_repository: Arc<dyn GlobalConfigRepository>,
        llm_repository: Arc<dyn LLMRepository>,
        jira_repository: Arc<dyn JiraRepository>,
        github_repository: Arc<dyn GitHubRepository>,
    ) -> Self {
        Self {
            config_repository,
            llm_repository,
            jira_repository,
            github_repository,
        }
    }
}

impl VerificationService for VerificationServiceImpl {
    /// 验证 Jira 配置
    fn verify_jira_config(&self) -> Result<JiraVerificationResult, ServiceError> {
        let user_info = self.jira_repository.get_user_info()?;
        let display_name = user_info.display_name;
        let account_id = user_info.account_id;

        let global_config = self.config_repository.load()?;
        let jira_settings = &global_config.jira;

        let config = JiraConfigInfo {
            email: jira_settings.email.clone(),
            service_address: jira_settings.service_address.clone(),
            api_token: jira_settings.api_token.mask(),
        };

        Ok(JiraVerificationResult {
            configured: true,
            config: Some(config),
            verification: Some(JiraVerificationStatus::Success {
                email: display_name,
                account_id,
            }),
        })
    }

    /// 验证 GitHub 配置
    fn verify_github_config(&self) -> Result<GitHubVerificationResult, ServiceError> {
        let global_config = self.config_repository.load()?;
        let github_settings = &global_config.github;

        if github_settings.accounts.is_empty() {
            return Ok(GitHubVerificationResult {
                configured: false,
                accounts: vec![],
                summary: GitHubVerificationSummary {
                    total_count: 0,
                    success_count: 0,
                    failed_accounts: vec![],
                },
            });
        }

        let mut accounts = Vec::new();
        let mut success_count = 0;
        let mut failed_accounts = Vec::new();

        // 验证 GitHub 认证
        let mut github_valid = false;
        let mut verification_error: Option<String> = None;

        match self.github_repository.get_user_info() {
            Ok(_) => {
                github_valid = true;
            }
            Err(err) => {
                verification_error = Some(err.to_string());
            }
        }

        for account in &github_settings.accounts {
            let is_current = github_settings.current == account.name;

            // 确定验证状态
            let verification_status = if github_valid {
                success_count += 1;
                "github".to_string()
            } else {
                failed_accounts.push(account.name.clone());
                String::new()
            };

            accounts.push(GitHubAccountInfo {
                name: account.name.clone(),
                is_current,
                email: account.email.clone(),
                token: account.api_token.mask(),
                verification_status: if is_current {
                    Some(verification_status)
                } else {
                    None
                },
                verification_error: if is_current {
                    verification_error.clone()
                } else {
                    None
                },
            });
        }

        let summary = GitHubVerificationSummary {
            total_count: accounts.len(),
            success_count,
            failed_accounts,
        };

        Ok(GitHubVerificationResult {
            configured: true,
            accounts,
            summary,
        })
    }

    /// 验证 LLM 配置
    fn verify_llm_config(&self) -> Result<LLMVerificationResult, ServiceError> {
        let global_config = self.config_repository.load()?;
        let llm_settings = &global_config.llm;

        if llm_settings.provider.is_empty() {
            return Ok(LLMVerificationResult {
                configured: false,
                config: None,
                verification: None,
            });
        }

        // 准备配置信息
        let current_provider = llm_settings.current_provider();
        let model = current_provider
            .model
            .clone()
            .unwrap_or_else(|| LLMSettings::default_model(&llm_settings.provider));

        let key = current_provider
            .key
            .as_ref()
            .map(|k| k.mask())
            .unwrap_or_else(|| "-".to_string());

        let language = if llm_settings.language.is_empty() {
            "en".to_string()
        } else {
            llm_settings.language.clone()
        };

        // 构建配置信息
        let config = LLMConfig {
            provider: llm_settings.provider.clone(),
            model: model.clone(),
            key: key.clone(),
            language: language.clone(),
        };

        let response = match self.llm_repository.verify_config() {
            Ok(response) => response,
            Err(err) => {
                // 提取原始错误消息，避免重复的 "LLM API 调用失败: " 前缀
                let reason = match &err {
                    domain::LLMError::ApiError(msg) => {
                        // 如果消息已经包含 "LLM API 调用失败: " 前缀，提取后面的部分
                        if let Some(stripped) = msg.strip_prefix("LLM API 调用失败: ") {
                            stripped.to_string()
                        } else {
                            msg.clone()
                        }
                    }
                    _ => err.to_string(),
                };
                return Ok(LLMVerificationResult {
                    configured: false,
                    config: Some(config),
                    verification: Some(LLMVerificationStatus::Failed {
                        reason,
                        details: vec![],
                    }),
                });
            }
        };

        Ok(LLMVerificationResult {
            configured: true,
            config: Some(config),
            verification: Some(LLMVerificationStatus::Success {
                test_response: response,
            }),
        })
    }

    /// 验证日志配置
    fn verify_log_config(&self) -> Result<LogVerificationResult, ServiceError> {
        let global_config = self.config_repository.load()?;
        let log_settings = &global_config.log;

        // 日志配置验证：检查配置是否存在
        // 如果 level 为 None，表示未配置，这是允许的
        let configured = log_settings.level.is_some();
        let config = if configured {
            Some(LogConfigInfo {
                output_folder_name: log_settings.get_output_folder_name(),
                download_base_dir: log_settings.download_base_dir.clone(),
                level: log_settings.level.clone(),
                enable_trace_console: log_settings.enable_trace_console.unwrap_or(false),
            })
        } else {
            None
        };

        Ok(LogVerificationResult { configured, config })
    }
}
