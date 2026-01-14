use crate::constants::{errors::http_client, git::check_errors, messages::log};
use crate::git::{GitCommit, GitRepo};
use crate::http::client::HttpClient;
use crate::http::{HttpMethod, RequestConfig};
use crate::prompt::{spinner, TableBuilder, TableStyle};
use crate::settings::paths::Paths;
use crate::settings::Settings;
use crate::settings::{GitHubAccountRow, JiraConfigRow, LLMConfigRow};
use crate::{br, error, info, success, warning};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use duct::cmd;
use serde_json::Value;
use std::time::Duration;

/// 环境检查结果
#[derive(Debug, Clone)]
struct EnvironmentCheckResult {
    /// Git 状态检查结果
    #[allow(dead_code)]
    git_status: GitStatusCheck,
    /// 网络连接检查结果
    #[allow(dead_code)]
    network: NetworkCheck,
    /// 配置文件权限警告（如果有）
    config_permissions_warning: Option<String>,
}

/// Git 状态检查结果
#[derive(Debug, Clone)]
struct GitStatusCheck {
    /// 是否在 Git 仓库中
    #[allow(dead_code)]
    is_git_repo: bool,
    /// Git 状态输出（如果有未提交的更改）
    #[allow(dead_code)]
    status_output: Option<String>,
}

/// 网络连接检查结果
#[derive(Debug, Clone)]
struct NetworkCheck {
    /// 是否成功
    #[allow(dead_code)]
    success: bool,
    /// 错误信息（如果失败）
    #[allow(dead_code)]
    error: Option<String>,
}

/// 环境检查命令
#[allow(dead_code)]
pub struct CheckCommand;

#[allow(dead_code)]
impl CheckCommand {
    /// 执行综合环境检查（类似 Go 版本的 check 命令）
    ///
    /// 包括：
    /// - 显示配置信息
    /// - 环境检查（Git、网络、配置文件权限）
    /// - 配置验证（Log、LLM、Jira、GitHub）
    pub fn run_all() -> Result<()> {
        info!("Starting check command");
        br!();

        // 1. 显示配置信息
        let workflow_config_path =
            Paths::workflow_config().map_err(|_| eyre!("Failed to get workflow config path"))?;

        if workflow_config_path.exists() {
            br!('=', 80, "Current Configuration");
            br!();
            info!("Workflow config: {:?}", workflow_config_path);
            br!();
        } else {
            warning!("Config file not found");
            br!();
        }

        // 2. 环境检查
        let env_result = Self::verify_environment()?;

        // 显示配置文件权限警告（如果有）
        if let Some(warning_msg) = env_result.config_permissions_warning {
            warning!("{}", warning_msg);
            br!();
        }

        // 3. 配置验证（如果配置文件存在）
        if workflow_config_path.exists() {
            let settings = Settings::load();

            // 检查是否有配置（检查关键配置项）
            let has_config = settings.jira.email.is_some()
                || settings.jira.api_token.is_some()
                || !settings.github.accounts.is_empty()
                || !settings.llm.openai.is_empty()
                || !settings.llm.deepseek.is_empty()
                || !settings.llm.proxy.is_empty();

            if has_config {
                // 逐个验证并展示结果
                Self::verify_and_display_all(&settings)?;
            }
        }

        br!();
        success!("All checks passed");
        Ok(())
    }

    /// 逐个验证并展示结果
    pub fn verify_and_display_all(settings: &Settings) -> Result<()> {
        // 1. 验证 Log 配置
        br!();
        info!("Verifying Log configuration...");
        let log_config = settings.log.get_config_info();
        info!("Log Output Folder Name: {}", log_config.output_folder_name);
        if let Some(ref dir) = log_config.download_base_dir {
            info!("Download Base Dir: {}", dir);
        }

        // 2. 验证 LLM 配置
        br!();
        let llm_result =
            spinner("Verifying LLM configuration...").with(|| settings.llm.verify())?;
        if llm_result.configured {
            if let Some(ref config) = llm_result.config {
                let config_rows = vec![LLMConfigRow {
                    provider: config.provider.clone(),
                    model: config.model.clone(),
                    key: config.key.clone(),
                    language: config.language.clone(),
                }];
                TableBuilder::from_tabled(config_rows).with_style(TableStyle::Modern).print()?;
            }
            if let Some(ref verification) = llm_result.verification {
                match verification {
                    crate::settings::LLMVerificationStatus::Success { test_response } => {
                        info!("  System prompt: You are a helpful assistant.");
                        info!("  User prompt: Say hello");
                        info!("  Response: {}", test_response);
                        success!("LLM verified successfully!");
                    }
                    crate::settings::LLMVerificationStatus::Failed { reason, details } => {
                        warning!("{}", reason);
                        for detail in details {
                            info!("  {}", detail);
                        }
                    }
                }
            }
        } else {
            info!("No LLM configuration found.");
        }

        // 3. 验证 Jira 配置
        br!();
        let jira_result =
            spinner("Verifying Jira configuration...").with(|| settings.jira.verify())?;
        if jira_result.configured {
            if let Some(ref config) = jira_result.config {
                let config_rows = vec![JiraConfigRow {
                    email: config.email.clone(),
                    service_address: config.service_address.clone(),
                    api_token: config.api_token.clone(),
                }];
                TableBuilder::from_tabled(config_rows).with_style(TableStyle::Modern).print()?;
            }
            if let Some(ref verification) = jira_result.verification {
                match verification {
                    crate::settings::JiraVerificationStatus::Success { email, account_id } => {
                        success!(
                            "Jira verified successfully! Email: {} (Account ID: {})",
                            email,
                            account_id
                        );
                    }
                    crate::settings::JiraVerificationStatus::Failed { reason, details } => {
                        warning!("{}", reason);
                        for detail in details {
                            info!("  {}", detail);
                        }
                    }
                }
            }
        } else {
            info!("No Jira configuration found.");
        }

        // 4. 验证 GitHub 配置
        br!();
        let github_result =
            spinner("Verifying GitHub configuration...").with(|| settings.github.verify())?;
        if github_result.configured {
            let account_rows: Vec<GitHubAccountRow> = github_result
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
                    status: if acc.is_current {
                        "Current".to_string()
                    } else {
                        "".to_string()
                    },
                    verification: acc.verification_status.clone(),
                })
                .collect();
            TableBuilder::from_tabled(account_rows).with_style(TableStyle::Modern).print()?;

            // 打印每个账号的详细错误信息（如果有）
            for account in &github_result.accounts {
                if let Some(ref error) = account.verification_error {
                    info!("  {}: {}", account.name, error);
                }
            }

            // 打印验证总结
            let summary = &github_result.summary;
            if summary.failed_accounts.is_empty() {
                br!();
                success!(
                    "All {} GitHub account(s) verified successfully!",
                    summary.total_count
                );
            } else {
                warning!(
                    "\nGitHub verification completed: {}/{} account(s) verified successfully",
                    summary.success_count,
                    summary.total_count
                );
                if !summary.failed_accounts.is_empty() {
                    info!("  Failed accounts: {}", summary.failed_accounts.join(", "));
                }
            }
        } else {
            info!("No GitHub configuration found.");
        }

        Ok(())
    }

    /// 执行完整的环境检查
    ///
    /// 包括：
    /// - Git 仓库状态检查
    /// - 网络连接检查（GitHub）
    /// - 配置文件权限检查
    fn verify_environment() -> Result<EnvironmentCheckResult> {
        info!("Running environment checks...");
        br!();

        // 1. 检查 Git 状态
        let git_status = Self::check_git_status()?;

        br!();

        // 2. 检查网络连接
        let network = Self::check_network()?;

        br!();

        // 3. 检查配置文件权限
        let config_permissions_warning = Self::check_config_permissions();

        Ok(EnvironmentCheckResult {
            git_status,
            network,
            config_permissions_warning,
        })
    }

    /// 检查 Git 仓库状态
    fn check_git_status() -> Result<GitStatusCheck> {
        info!("[1/2] Checking Git repository status...");
        if !GitRepo::is_git_repo() {
            error!("Not in a Git repository");
            return Err(color_eyre::eyre::eyre!("{}", check_errors::NOT_GIT_REPO));
        }

        let git_output = GitCommit::status().wrap_err("Failed to check git status")?;
        if git_output.trim().is_empty() {
            success!("Git repository is clean (no uncommitted changes)");
            Ok(GitStatusCheck {
                is_git_repo: true,
                status_output: None,
            })
        } else {
            info!("Git status:\n{}", git_output);
            Ok(GitStatusCheck {
                is_git_repo: true,
                status_output: Some(git_output),
            })
        }
    }

    /// 检查网络连接（GitHub）
    fn check_network() -> Result<NetworkCheck> {
        info!("[2/2] Checking network connection to GitHub...");
        let client = HttpClient::global().wrap_err(http_client::CREATE_CLIENT_FAILED)?;
        let config = RequestConfig::<Value, Value>::new().timeout(Duration::from_secs(10));
        match client.stream(HttpMethod::Get, crate::git::github::BASE, config) {
            Ok(response) => {
                if response.status().is_success() {
                    success!("GitHub network is available");
                    Ok(NetworkCheck {
                        success: true,
                        error: None,
                    })
                } else {
                    let error_msg = format!(
                        "GitHub network check failed (status: {})",
                        response.status()
                    );
                    error!("{}", error_msg);
                    Err(color_eyre::eyre::eyre!("Network check failed"))
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to check network connection: {}", e);
                error!("{}", error_msg);
                error!(
                    "  This might be due to network issues, proxy settings, or firewall restrictions"
                );
                Err(color_eyre::eyre::eyre!("Network check failed: {}", e))
            }
        }
    }

    /// 检查配置文件权限
    ///
    /// 返回警告信息（如果有），否则返回 `None`。
    fn check_config_permissions() -> Option<String> {
        Settings::check_permissions()
    }

    /// 执行代码质量检查（Lint）
    ///
    /// 通过调用 `make lint` 来执行完整的代码质量检查，包括：
    /// - 代码格式检查（cargo fmt --check）
    /// - Clippy 检查（cargo clippy -- -D warnings）
    /// - 编译检查（cargo check）
    ///
    /// 这样可以复用 Makefile 中定义的 lint 规则，保持一致性。
    pub fn run_lint() -> Result<()> {
        info!("Running code quality checks (Lint)...");
        br!();

        // 检查 make 命令是否可用（跨平台检查）
        let make_available = if cfg!(target_os = "windows") {
            cmd("where", &["make"]).run().is_ok()
        } else {
            cmd("which", &["make"]).run().is_ok()
        };

        if !make_available {
            error!("make command is not available");
            error!("Please install make or run lint checks manually:");
            error!("  cargo fmt --check");
            error!("  cargo clippy -- -D warnings");
            error!("  cargo check");
            color_eyre::eyre::bail!("make command not found");
        }

        // 使用 make lint 执行检查
        info!("Running 'make lint'...");
        let lint_output = cmd("make", &["lint"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .wrap_err("Failed to run make lint")?;

        if !lint_output.status.success() {
            let stderr = String::from_utf8_lossy(&lint_output.stderr);
            let stdout = String::from_utf8_lossy(&lint_output.stdout);
            error!("Lint check failed");
            if !stderr.is_empty() {
                error!("{}", stderr);
            }
            if !stdout.is_empty() {
                error!("{}", stdout);
            }
            error!("Run 'make fix' to auto-fix some issues, or fix them manually");
            color_eyre::eyre::bail!("Lint check failed");
        }

        // 输出 make lint 的结果（成功时）
        let stdout = String::from_utf8_lossy(&lint_output.stdout);
        if !stdout.is_empty() {
            info!("{}", stdout);
        }

        success!("All lint checks passed");
        Ok(())
    }

    /// 执行测试检查
    ///
    /// 通过调用 `cargo test` 来运行所有测试，确保代码功能正常。
    pub fn run_test() -> Result<()> {
        info!("Running tests...");
        br!();

        // 运行 cargo test
        info!("Running 'cargo test'...");
        let test_output = cmd("cargo", &["test", "--verbose"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .wrap_err("Failed to run cargo test")?;

        if !test_output.status.success() {
            let stderr = String::from_utf8_lossy(&test_output.stderr);
            let stdout = String::from_utf8_lossy(&test_output.stdout);
            error!("{}", log::TESTS_FAILED);
            if !stderr.is_empty() {
                error!("{}", stderr);
            }
            if !stdout.is_empty() {
                error!("{}", stdout);
            }
            error!("Please fix the failing tests before merging");
            color_eyre::eyre::bail!("{}", log::TESTS_FAILED);
        }

        // 输出测试结果（成功时）
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        if !stdout.is_empty() {
            // 只显示测试摘要，避免输出过多
            let lines: Vec<&str> = stdout.lines().collect();
            let summary_start = lines
                .iter()
                .rposition(|line| line.contains("test result:") || line.contains("running"));

            if let Some(start) = summary_start {
                let summary: String = lines[start..].join("\n");
                info!("{}", summary);
            } else {
                // 如果没有找到摘要，显示最后几行
                let last_lines: Vec<&str> = lines.iter().rev().take(10).rev().copied().collect();
                if !last_lines.is_empty() {
                    info!("{}", last_lines.join("\n"));
                }
            }
        }

        success!("All tests passed");
        Ok(())
    }
}
