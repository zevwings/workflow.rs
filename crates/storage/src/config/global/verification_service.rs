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

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        config::global::{github::config::GitHubAccount, jira::config::JiraSettings},
        AttachmentDownloadResult, GitHubError, GitHubUser, GlobalConfig, JiraAttachment, JiraError,
        JiraIssue, JiraStatusConfig, JiraUser, LLMError, PrContent, PullRequestContent,
        PullRequestInfo, PullRequestReword, PullRequestSummary,
    };
    use std::path::Path;

    #[derive(Clone)]
    struct MockGlobalConfigRepository {
        config: GlobalConfig,
    }

    impl GlobalConfigRepository for MockGlobalConfigRepository {
        fn load(&self) -> Result<GlobalConfig, ServiceError> {
            Ok(self.config.clone())
        }

        fn save(&self, _settings: &GlobalConfig) -> Result<(), ServiceError> {
            Ok(())
        }

        fn check_permissions(&self) -> Option<String> {
            None
        }
    }

    struct MockJiraRepository {
        user: JiraUser,
    }

    impl JiraRepository for MockJiraRepository {
        fn get_user_info(&self) -> Result<JiraUser, JiraError> {
            Ok(self.user.clone())
        }

        fn get_issue_info(&self, _issue_id: &str) -> Result<JiraIssue, JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn update_issue_status(&self, _issue_id: &str, _status: &str) -> Result<(), JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn add_comment(&self, _issue_id: &str, _comment: &str) -> Result<(), JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn get_attachments(&self, _issue_id: &str) -> Result<Vec<JiraAttachment>, JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn download_attachments(
            &self,
            _issue_id: &str,
            _base_dir: &Path,
        ) -> Result<AttachmentDownloadResult, JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn clean_attachments(&self, _jira_id: Option<&str>) -> Result<(), JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn get_project_statuses(&self, _project: &str) -> Result<Vec<String>, JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn write_status_config(&self, _config: &JiraStatusConfig) -> Result<(), JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn read_pull_request_created_status(
            &self,
            _jira_ticket: &str,
        ) -> Result<Option<String>, JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }

        fn read_pull_request_merged_status(
            &self,
            _jira_ticket: &str,
        ) -> Result<Option<String>, JiraError> {
            Err(JiraError::ApiError("not implemented".to_string()))
        }
    }

    struct MockGitHubRepository {
        user: GitHubUser,
    }

    impl GitHubRepository for MockGitHubRepository {
        fn create_pull_request(
            &self,
            _title: &str,
            _body: &str,
            _source_branch: &str,
            _target_branch: &str,
        ) -> Result<String, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_pull_request(&self, _pr_id: &str) -> Result<PullRequestInfo, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn merge_pull_request(&self, _pr_id: &str, _force: bool) -> Result<(), GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_user_info(&self) -> Result<GitHubUser, GitHubError> {
            Ok(self.user.clone())
        }

        fn close_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn list_pull_requests(
            &self,
            _state: Option<&str>,
            _limit: Option<usize>,
        ) -> Result<Vec<PullRequestInfo>, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn update_pull_request(
            &self,
            _pr_id: &str,
            _title: Option<&str>,
            _body: Option<&str>,
        ) -> Result<(), GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn add_comment(&self, _pr_id: &str, _comment: &str) -> Result<(), GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn approve_pull_request(&self, _pr_id: &str) -> Result<(), GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_pr_diff(&self, _pr_id: &str) -> Result<String, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_pull_request_info(&self, _pr_id: &str) -> Result<String, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_pull_request_url(&self, _pr_id: &str) -> Result<String, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_pull_request_title(&self, _pr_id: &str) -> Result<String, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_pull_request_body(&self, _pr_id: &str) -> Result<Option<String>, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_pull_request_status(
            &self,
            _pr_id: &str,
        ) -> Result<(String, bool, Option<String>), GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn update_pr_base(&self, _pr_id: &str, _new_base: &str) -> Result<(), GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }

        fn get_current_branch_pull_request(
            &self,
            _current_branch: &str,
        ) -> Result<Option<String>, GitHubError> {
            Err(GitHubError::ApiError("not implemented".to_string()))
        }
    }

    struct MockLLMRepository {
        verify_result: Result<String, LLMError>,
    }

    impl LLMRepository for MockLLMRepository {
        fn verify_config(&self) -> Result<String, LLMError> {
            self.verify_result.clone()
        }

        fn generate_branch_name(
            &self,
            _title: Option<&str>,
            _exists_branches: Option<Vec<String>>,
        ) -> Result<String, LLMError> {
            Err(LLMError::ApiError("not implemented".to_string()))
        }

        fn generate_pr_content(
            &self,
            _branch_name: &str,
            _commits: &[String],
        ) -> Result<PrContent, LLMError> {
            Err(LLMError::ApiError("not implemented".to_string()))
        }

        fn generate_commit_message(&self, _changes: &str) -> Result<String, LLMError> {
            Err(LLMError::ApiError("not implemented".to_string()))
        }

        fn translate_to_english(&self, _text: &str) -> Result<String, LLMError> {
            Err(LLMError::ApiError("not implemented".to_string()))
        }

        fn create_pr_content(
            &self,
            _commit_title: &str,
            _exists_branches: Option<Vec<String>>,
            _git_diff: Option<String>,
        ) -> Result<PullRequestContent, LLMError> {
            Err(LLMError::ApiError("not implemented".to_string()))
        }

        fn reword_pr(
            &self,
            _pr_diff: &str,
            _current_title: Option<&str>,
        ) -> Result<PullRequestReword, LLMError> {
            Err(LLMError::ApiError("not implemented".to_string()))
        }

        fn summarize_pr(
            &self,
            _pr_title: &str,
            _pr_diff: &str,
        ) -> Result<PullRequestSummary, LLMError> {
            Err(LLMError::ApiError("not implemented".to_string()))
        }

        fn summarize_file_change(
            &self,
            _file_path: &str,
            _file_diff: &str,
        ) -> Result<String, LLMError> {
            Err(LLMError::ApiError("not implemented".to_string()))
        }
    }

    #[test]
    fn test_verify_jira_config_success() {
        let config = GlobalConfig {
            jira: JiraSettings {
                email: "user@example.com".to_string(),
                api_token: "token".to_string(),
                service_address: "https://jira.example.com".to_string(),
            },
            ..GlobalConfig::default()
        };

        let service = VerificationServiceImpl::new(
            Arc::new(MockGlobalConfigRepository { config }),
            Arc::new(MockLLMRepository {
                verify_result: Ok("ok".to_string()),
            }),
            Arc::new(MockJiraRepository {
                user: JiraUser {
                    display_name: "User".to_string(),
                    account_id: "123".to_string(),
                },
            }),
            Arc::new(MockGitHubRepository {
                user: GitHubUser {
                    login: "me".to_string(),
                    name: Some("Me".to_string()),
                    email: Some("me@example.com".to_string()),
                },
            }),
        );

        let result = service.verify_jira_config().unwrap();
        assert!(result.configured);
        match result.verification {
            Some(JiraVerificationStatus::Success { .. }) => {}
            _ => panic!("expected success"),
        }
    }

    #[test]
    fn test_verify_github_config_no_accounts() {
        let config = GlobalConfig::default();
        let service = VerificationServiceImpl::new(
            Arc::new(MockGlobalConfigRepository { config }),
            Arc::new(MockLLMRepository {
                verify_result: Ok("ok".to_string()),
            }),
            Arc::new(MockJiraRepository {
                user: JiraUser {
                    display_name: "User".to_string(),
                    account_id: "123".to_string(),
                },
            }),
            Arc::new(MockGitHubRepository {
                user: GitHubUser {
                    login: "me".to_string(),
                    name: Some("Me".to_string()),
                    email: Some("me@example.com".to_string()),
                },
            }),
        );

        let result = service.verify_github_config().unwrap();
        assert!(!result.configured);
        assert!(result.accounts.is_empty());
    }

    #[test]
    fn test_verify_llm_config_failed() {
        let config = GlobalConfig {
            llm: LLMSettings {
                provider: "openai".to_string(),
                language: "en".to_string(),
                openai: domain::config::global::llm::config::LLMProviderSettings {
                    key: Some("key".to_string()),
                    ..domain::config::global::llm::config::LLMProviderSettings::default()
                },
                deepseek: domain::config::global::llm::config::LLMProviderSettings::default(),
                proxy: domain::config::global::llm::config::LLMProviderSettings::default(),
            },
            ..GlobalConfig::default()
        };

        let service = VerificationServiceImpl::new(
            Arc::new(MockGlobalConfigRepository { config }),
            Arc::new(MockLLMRepository {
                verify_result: Err(LLMError::ApiError("LLM API 调用失败: bad".to_string())),
            }),
            Arc::new(MockJiraRepository {
                user: JiraUser {
                    display_name: "User".to_string(),
                    account_id: "123".to_string(),
                },
            }),
            Arc::new(MockGitHubRepository {
                user: GitHubUser {
                    login: "me".to_string(),
                    name: Some("Me".to_string()),
                    email: Some("me@example.com".to_string()),
                },
            }),
        );

        let result = service.verify_llm_config().unwrap();
        assert!(!result.configured);
        match result.verification {
            Some(LLMVerificationStatus::Failed { reason, .. }) => {
                assert_eq!(reason, "bad");
            }
            _ => panic!("expected failure"),
        }
    }

    #[test]
    fn test_verify_log_config_configured() {
        let config = GlobalConfig {
            log: domain::config::global::log::config::LogSettings {
                level: Some("info".to_string()),
                ..domain::config::global::log::config::LogSettings::default()
            },
            ..GlobalConfig::default()
        };

        let service = VerificationServiceImpl::new(
            Arc::new(MockGlobalConfigRepository { config }),
            Arc::new(MockLLMRepository {
                verify_result: Ok("ok".to_string()),
            }),
            Arc::new(MockJiraRepository {
                user: JiraUser {
                    display_name: "User".to_string(),
                    account_id: "123".to_string(),
                },
            }),
            Arc::new(MockGitHubRepository {
                user: GitHubUser {
                    login: "me".to_string(),
                    name: Some("Me".to_string()),
                    email: Some("me@example.com".to_string()),
                },
            }),
        );

        let result = service.verify_log_config().unwrap();
        assert!(result.configured);
        assert!(result.config.is_some());
    }

    #[test]
    fn test_verify_github_config_with_account_success() {
        let config = GlobalConfig {
            github: domain::config::global::github::config::GitHubSettings {
                accounts: vec![GitHubAccount {
                    name: "work".to_string(),
                    email: "work@example.com".to_string(),
                    api_token: "token".to_string(),
                }],
                current: "work".to_string(),
            },
            ..GlobalConfig::default()
        };

        let service = VerificationServiceImpl::new(
            Arc::new(MockGlobalConfigRepository { config }),
            Arc::new(MockLLMRepository {
                verify_result: Ok("ok".to_string()),
            }),
            Arc::new(MockJiraRepository {
                user: JiraUser {
                    display_name: "User".to_string(),
                    account_id: "123".to_string(),
                },
            }),
            Arc::new(MockGitHubRepository {
                user: GitHubUser {
                    login: "me".to_string(),
                    name: Some("Me".to_string()),
                    email: Some("me@example.com".to_string()),
                },
            }),
        );

        let result = service.verify_github_config().unwrap();
        assert!(result.configured);
        assert_eq!(result.summary.total_count, 1);
        assert_eq!(result.summary.success_count, 1);
    }
}
