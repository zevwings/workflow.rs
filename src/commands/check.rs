use crate::{log_error, log_info, log_success, Git};
use anyhow::{Context, Result};
use duct::cmd;

/// 检查工具命令
pub struct CheckCommand;

impl CheckCommand {
    /// 执行综合检查
    #[allow(dead_code)]
    pub fn run_all() -> Result<()> {
        // 1. 检查 Git 状态
        Self::check_git_status()?;

        // 2. 检查网络连接
        Self::check_network()?;

        log_success!("All checks passed");
        Ok(())
    }

    /// 检查 Git 状态
    pub fn check_git_status() -> Result<()> {
        if !Git::is_git_repo() {
            anyhow::bail!("Not in a Git repository");
        }

        let output = Git::status().context("Failed to check git status")?;

        log_info!("{}", output);
        Ok(())
    }

    /// 检查网络连接（GitHub）
    pub fn check_network() -> Result<()> {
        log_info!("Checking network connection to GitHub...");
        let output = cmd("curl", &["-IsSf", "--max-time", "10", "https://github.com"])
            .stdout_null()
            .stderr_null()
            .run();

        match output {
            Ok(result) => {
                if result.status.success() {
                    log_success!("GitHub network is available");
                    Ok(())
                } else {
                    log_error!("GitHub network check failed (curl returned non-zero exit code)");
                    anyhow::bail!("Network check failed");
                }
            }
            Err(e) => {
                log_error!("Failed to check network connection: {}", e);
                log_error!(
                    "This might be due to network issues, proxy settings, or firewall restrictions"
                );
                anyhow::bail!("Network check failed: {}", e);
            }
        }
    }
}
