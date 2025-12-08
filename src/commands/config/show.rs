//! 配置查看命令
//! 显示当前的 TOML 配置文件

use crate::base::settings::paths::Paths;
use crate::base::settings::settings::Settings;
use crate::base::util::table::{TableBuilder, TableStyle};
use crate::{log_break, log_info, log_message, log_success, log_warning};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tabled::Tabled;

/// 配置查看命令
pub struct ConfigCommand;

impl ConfigCommand {
    /// 显示当前配置（从 TOML 文件读取）
    pub fn show() -> Result<()> {
        log_break!('=', 40, "Current Configuration");
        log_break!();

        // 显示配置文件路径
        let workflow_config_path = Paths::workflow_config()
            .map_err(|_| anyhow::anyhow!("Failed to get workflow config path"))?;

        log_info!("Workflow config: {:?}\n", workflow_config_path);

        // 检查配置文件权限
        if let Some(warning) = Settings::check_permissions() {
            log_warning!("{}", warning);
        }

        // 从 TOML 文件加载配置（使用 load() 获取最新配置，避免 OnceLock 缓存问题）
        let settings = Settings::load();

        // 检查是否有配置
        if Self::is_empty_config(&settings) {
            log_warning!("  No configuration found!");
            log_info!("   Run 'workflow setup' to initialize configuration.");
            return Ok(());
        }

        // 显示所有配置
        log_break!();
        log_break!('-', 100, "Configuration");

        // 创建 spinner 显示验证进度
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.white} {msg}")
                .unwrap(),
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message("Verifying configurations...");

        let result = settings.verify()?;

        // 完成 spinner
        spinner.finish_and_clear();

        Self::print_verification_result(&result);

        Ok(())
    }

    /// 打印验证结果
    pub fn print_verification_result(result: &crate::base::settings::settings::VerificationResult) {
        // 打印日志配置
        log_message!("Log Output Folder Name: {}", result.log.output_folder_name);
        if let Some(ref dir) = result.log.download_base_dir {
            log_message!("Download Base Dir: {}", dir);
        }

        // 打印 LLM 配置
        log_break!();
        log_info!("LLM Configuration");
        #[derive(Tabled)]
        struct LLMConfigRow {
            #[tabled(rename = "Provider")]
            provider: String,
            #[tabled(rename = "Model")]
            model: String,
            #[tabled(rename = "Key")]
            key: String,
            #[tabled(rename = "Output Language")]
            language: String,
        }
        let config_rows = vec![LLMConfigRow {
            provider: result.llm.provider.clone(),
            model: result.llm.model.clone(),
            key: result.llm.key.clone(),
            language: result.llm.language.clone(),
        }];
        TableBuilder::new(config_rows)
            .with_style(TableStyle::Modern)
            .print();

        // 打印 Codeup 配置
        if result.codeup.project_id.is_some()
            || result.codeup.csrf_token.is_some()
            || result.codeup.cookie.is_some()
        {
            if let Some(id) = result.codeup.project_id {
                log_message!("\nCodeup Project ID: {}", id);
            }
            if let Some(ref token) = result.codeup.csrf_token {
                log_message!("Codeup CSRF Token: {}", token);
            }
            if let Some(ref cookie) = result.codeup.cookie {
                log_message!("Codeup Cookie: {}", cookie);
            }
        }

        // 打印 Jira 配置
        if result.jira.configured {
            log_break!();
            log_info!("Verifying Jira configuration...");
            if let Some(ref config) = result.jira.config {
                #[derive(Tabled)]
                struct JiraConfigRow {
                    #[tabled(rename = "Email")]
                    email: String,
                    #[tabled(rename = "Service Address")]
                    service_address: String,
                    #[tabled(rename = "API Token")]
                    api_token: String,
                }
                let config_rows = vec![JiraConfigRow {
                    email: config.email.clone(),
                    service_address: config.service_address.clone(),
                    api_token: config.api_token.clone(),
                }];
                TableBuilder::new(config_rows)
                    .with_style(TableStyle::Modern)
                    .print();
            }
            if let Some(ref verification) = result.jira.verification {
                match verification {
                    crate::base::settings::settings::JiraVerificationStatus::Success {
                        email,
                        account_id,
                    } => {
                        log_success!(
                            "Jira verified successfully! Email: {} (Account ID: {})",
                            email,
                            account_id
                        );
                    }
                    crate::base::settings::settings::JiraVerificationStatus::Failed {
                        reason,
                        details,
                    } => {
                        log_warning!("{}", reason);
                        for detail in details {
                            log_message!("  {}", detail);
                        }
                    }
                }
            }
        }

        // 打印 GitHub 配置
        if result.github.configured {
            log_break!();
            log_info!("Verifying GitHub configuration...");
            #[derive(Tabled)]
            struct GitHubAccountRow {
                #[tabled(rename = "Name")]
                name: String,
                #[tabled(rename = "Email")]
                email: String,
                #[tabled(rename = "API Token")]
                token: String,
                #[tabled(rename = "Branch Prefix")]
                prefix: String,
                #[tabled(rename = "Status")]
                status: String,
                #[tabled(rename = "Verification")]
                verification: String,
            }
            let account_rows: Vec<GitHubAccountRow> = result
                .github
                .accounts
                .iter()
                .map(|acc| GitHubAccountRow {
                    name: if acc.is_current {
                        format!("{} (current)", acc.name)
                    } else {
                        acc.name.clone()
                    },
                    email: acc.email.clone(),
                    token: acc.token.clone(),
                    prefix: acc.branch_prefix.clone().unwrap_or_else(|| "-".to_string()),
                    status: if acc.is_current {
                        "Current".to_string()
                    } else {
                        "".to_string()
                    },
                    verification: acc.verification_status.clone(),
                })
                .collect();
            TableBuilder::new(account_rows)
                .with_style(TableStyle::Modern)
                .print();

            // 打印每个账号的详细错误信息（如果有）
            for account in &result.github.accounts {
                if let Some(ref error) = account.verification_error {
                    log_message!("  {}: {}", account.name, error);
                }
            }

            // 打印验证总结
            let summary = &result.github.summary;
            if summary.failed_accounts.is_empty() {
                log_break!();
                log_success!(
                    "All {} GitHub account(s) verified successfully!",
                    summary.total_count
                );
            } else {
                log_warning!(
                    "\nGitHub verification completed: {}/{} account(s) verified successfully",
                    summary.success_count,
                    summary.total_count
                );
                if !summary.failed_accounts.is_empty() {
                    log_message!("  Failed accounts: {}", summary.failed_accounts.join(", "));
                }
            }
        }
    }

    /// 检查配置是否为空
    fn is_empty_config(settings: &Settings) -> bool {
        settings.jira.email.is_none()
            && settings.jira.api_token.is_none()
            && settings.github.accounts.is_empty()
            && settings.codeup.project_id.is_none()
            && settings.llm.url.is_none()
            && settings.llm.key.is_none()
    }
}
