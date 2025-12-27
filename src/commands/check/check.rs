use crate::base::constants::{errors::http_client, git::check_errors, messages::log};
use crate::base::http::client::HttpClient;
use crate::base::http::{HttpMethod, RequestConfig};
use crate::git::{GitCommit, GitRepo};
use crate::{log_break, log_error, log_info, log_message, log_success};
use color_eyre::{eyre::WrapErr, Result};
use duct::cmd;
use serde_json::Value;
use std::time::Duration;

/// 环境检查命令
pub struct CheckCommand;

impl CheckCommand {
    /// 执行综合环境检查
    ///
    /// 检查 Git 仓库状态和到 GitHub 的网络连接。
    pub fn run_all() -> Result<()> {
        log_message!("Running environment checks...");
        log_break!();

        log_message!("[1/2] Checking Git repository status...");
        if !GitRepo::is_git_repo() {
            log_error!("Not in a Git repository");
            color_eyre::eyre::bail!("{}", check_errors::NOT_GIT_REPO);
        }

        let git_output = GitCommit::status().wrap_err("Failed to check git status")?;
        if git_output.trim().is_empty() {
            log_success!("Git repository is clean (no uncommitted changes)");
        } else {
            log_info!("Git status:\n{}", git_output);
        }

        log_break!();

        log_message!("[2/2] Checking network connection to GitHub...");
        let client = HttpClient::global().wrap_err(http_client::CREATE_CLIENT_FAILED)?;
        let config = RequestConfig::<Value, Value>::new().timeout(Duration::from_secs(10));

        // 支持从环境变量读取 GitHub URL（用于测试 Mock 服务器）
        // 优先级：GITHUB_BASE_URL > GITHUB_API_URL > 硬编码 BASE
        let github_url = std::env::var("GITHUB_BASE_URL")
            .or_else(|_| std::env::var("GITHUB_API_URL"))
            .unwrap_or_else(|_| crate::git::github::BASE.to_string());

        match client.stream(HttpMethod::Get, &github_url, config) {
            Ok(response) => {
                if response.status().is_success() {
                    log_success!("GitHub network is available");
                } else {
                    log_error!(
                        "GitHub network check failed (status: {})",
                        response.status()
                    );
                    color_eyre::eyre::bail!("Network check failed");
                }
            }
            Err(e) => {
                log_error!("Failed to check network connection: {}", e);
                log_error!(
                "  This might be due to network issues, proxy settings, or firewall restrictions"
            );
                color_eyre::eyre::bail!("Network check failed: {}", e);
            }
        }

        log_break!();
        log_success!("All checks passed");
        Ok(())
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
        log_message!("Running code quality checks (Lint)...");
        log_break!();

        // 检查 make 命令是否可用（跨平台检查）
        let make_available = if cfg!(target_os = "windows") {
            cmd("where", &["make"]).run().is_ok()
        } else {
            cmd("which", &["make"]).run().is_ok()
        };

        if !make_available {
            log_error!("make command is not available");
            log_error!("Please install make or run lint checks manually:");
            log_error!("  cargo fmt --check");
            log_error!("  cargo clippy -- -D warnings");
            log_error!("  cargo check");
            color_eyre::eyre::bail!("make command not found");
        }

        // 使用 make lint 执行检查
        log_message!("Running 'make lint'...");
        let lint_output = cmd("make", &["lint"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .wrap_err("Failed to run make lint")?;

        if !lint_output.status.success() {
            let stderr = String::from_utf8_lossy(&lint_output.stderr);
            let stdout = String::from_utf8_lossy(&lint_output.stdout);
            log_error!("Lint check failed");
            if !stderr.is_empty() {
                log_error!("{}", stderr);
            }
            if !stdout.is_empty() {
                log_error!("{}", stdout);
            }
            log_error!("Run 'make fix' to auto-fix some issues, or fix them manually");
            color_eyre::eyre::bail!("Lint check failed");
        }

        // 输出 make lint 的结果（成功时）
        let stdout = String::from_utf8_lossy(&lint_output.stdout);
        if !stdout.is_empty() {
            log_info!("{}", stdout);
        }

        log_success!("All lint checks passed");
        Ok(())
    }

    /// 执行测试检查
    ///
    /// 通过调用 `cargo test` 来运行所有测试，确保代码功能正常。
    pub fn run_test() -> Result<()> {
        log_message!("Running tests...");
        log_break!();

        // 运行 cargo test
        log_message!("Running 'cargo test'...");
        let test_output = cmd("cargo", &["test", "--verbose"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .wrap_err("Failed to run cargo test")?;

        if !test_output.status.success() {
            let stderr = String::from_utf8_lossy(&test_output.stderr);
            let stdout = String::from_utf8_lossy(&test_output.stdout);
            log_error!("{}", log::TESTS_FAILED);
            if !stderr.is_empty() {
                log_error!("{}", stderr);
            }
            if !stdout.is_empty() {
                log_error!("{}", stdout);
            }
            log_error!("Please fix the failing tests before merging");
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
                log_info!("{}", summary);
            } else {
                // 如果没有找到摘要，显示最后几行
                let last_lines: Vec<&str> = lines.iter().rev().take(10).rev().copied().collect();
                if !last_lines.is_empty() {
                    log_info!("{}", last_lines.join("\n"));
                }
            }
        }

        log_success!("All tests passed");
        Ok(())
    }
}
