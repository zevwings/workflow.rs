use crate::base::http::client::HttpClient;
use crate::base::http::{HttpMethod, RequestConfig};
use crate::git::{GitCommit, GitRepo};
use crate::{log_break, log_error, log_info, log_message, log_success};
use anyhow::{Context, Result};
use duct::cmd;
use serde_json::Value;
use std::time::Duration;

/// 环境检查命令
#[allow(dead_code)]
pub struct CheckCommand;

#[allow(dead_code)]
impl CheckCommand {
    /// 执行综合环境检查
    ///
    /// 检查 Git 仓库状态和到 GitHub 的网络连接。
    pub fn run_all() -> Result<()> {
        log_message!("Running environment checks...");
        log_break!();

        // 1. 检查 Git 状态
        log_message!("[1/2] Checking Git repository status...");
        if !GitRepo::is_git_repo() {
            log_error!("Not in a Git repository");
            anyhow::bail!("Git check failed: Not in a Git repository");
        }

        let git_output = GitCommit::status().context("Failed to check git status")?;
        if git_output.trim().is_empty() {
            log_success!("Git repository is clean (no uncommitted changes)");
        } else {
            log_info!("Git status:\n{}", git_output);
        }

        log_break!();

        // 2. 检查网络连接
        log_message!("[2/2] Checking network connection to GitHub...");
        let client = HttpClient::global().context("Failed to create HTTP client")?;
        let config = RequestConfig::<Value, Value>::new().timeout(Duration::from_secs(10));
        match client.stream(HttpMethod::Get, "https://github.com", config) {
            Ok(response) => {
                if response.status().is_success() {
                    log_success!("GitHub network is available");
                } else {
                    log_error!(
                        "GitHub network check failed (status: {})",
                        response.status()
                    );
                    anyhow::bail!("Network check failed");
                }
            }
            Err(e) => {
                log_error!("Failed to check network connection: {}", e);
                log_error!(
                "  This might be due to network issues, proxy settings, or firewall restrictions"
            );
                anyhow::bail!("Network check failed: {}", e);
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
            anyhow::bail!("make command not found");
        }

        // 使用 make lint 执行检查
        log_message!("Running 'make lint'...");
        let lint_output = cmd("make", &["lint"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .context("Failed to run make lint")?;

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
            anyhow::bail!("Lint check failed");
        }

        // 输出 make lint 的结果（成功时）
        let stdout = String::from_utf8_lossy(&lint_output.stdout);
        if !stdout.is_empty() {
            log_info!("{}", stdout);
        }

        log_success!("All lint checks passed");
        Ok(())
    }
}
