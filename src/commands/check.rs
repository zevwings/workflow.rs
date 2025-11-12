use crate::{Git, HttpClient, log_error, log_info, log_break, log_success};
use anyhow::{Context, Result};

/// 执行综合环境检查
///
/// 检查 Git 仓库状态和到 GitHub 的网络连接。
pub fn run_all() -> Result<()> {
    log_info!("Running environment checks...");
    log_break!();

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

    log_break!();

    // 2. 检查网络连接
    log_info!("[2/2] Checking network connection to GitHub...");
    let client = HttpClient::new().context("Failed to create HTTP client")?;
    match client
        .client()
        .get("https://github.com")
        .timeout(std::time::Duration::from_secs(10))
        .send()
    {
        Ok(response) => {
            if response.status().is_success() {
                log_success!("✓ GitHub network is available");
            } else {
                log_error!("✗ GitHub network check failed (status: {})", response.status());
                anyhow::bail!("Network check failed");
            }
        }
        Err(e) => {
            log_error!("✗ Failed to check network connection: {}", e);
            log_error!("  This might be due to network issues, proxy settings, or firewall restrictions");
            anyhow::bail!("Network check failed: {}", e);
        }
    }

    log_break!();
    log_success!("All checks passed ✓");
    Ok(())
}
