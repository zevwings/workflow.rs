use crate::{log_error, log_info, log_success, Git};
use anyhow::{Context, Result};
use duct::cmd;

/// 检查工具命令
pub struct CheckCommand;

impl CheckCommand {
    /// 执行综合检查
    pub fn run_all() -> Result<()> {
        log_info!("Running environment checks...");
        println!();

        // 1. 检查 Git 状态
        log_info!("[1/2] Checking Git repository status...");
        if !Git::is_git_repo() {
            log_error!("✗ Not in a Git repository");
            anyhow::bail!("Git check failed: Not in a Git repository");
        }

        let git_output = Git::status().context("Failed to check git status")?;
        if git_output.trim().is_empty() {
            log_success!("✓ Git repository is clean (no uncommitted changes)");
        } else {
            log_info!("Git status:\n{}", git_output);
        }

        println!();

        // 2. 检查网络连接
        log_info!("[2/2] Checking network connection to GitHub...");
        let network_result = cmd("curl", &["-IsSf", "--max-time", "10", "https://github.com"])
            .stdout_null()
            .stderr_null()
            .run();

        match network_result {
            Ok(result) => {
                if result.status.success() {
                    log_success!("✓ GitHub network is available");
                } else {
                    log_error!("✗ GitHub network check failed (curl returned non-zero exit code)");
                    anyhow::bail!("Network check failed");
                }
            }
            Err(e) => {
                log_error!("✗ Failed to check network connection: {}", e);
                log_error!("  This might be due to network issues, proxy settings, or firewall restrictions");
                anyhow::bail!("Network check failed: {}", e);
            }
        }

        println!();
        log_success!("All checks passed ✓");
        Ok(())
    }
}
