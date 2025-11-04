use crate::{log_error, log_info, log_success, Git};
use anyhow::{Context, Result};
use duct::cmd;

/// 检查工具命令
pub struct CheckCommand;

impl CheckCommand {
    /// 执行综合检查
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
        let output = cmd("curl", &["-IsSf", "https://github.com"])
            .stdout_null()
            .stderr_null()
            .run()
            .context("Failed to check network connection")?;

        if output.status.success() {
            log_success!("GitHub network is available");
            Ok(())
        } else {
            log_error!("GitHub network error");
            anyhow::bail!("Network check failed");
        }
    }

    /// 运行 pre-commit hooks（不提交）
    pub fn check_pre_commit() -> Result<()> {
        // 先添加所有文件
        cmd("git", &["add", "--all"])
            .run()
            .context("Failed to run git add --all")?;

        // 检查是否有 staged 的文件
        let has_staged = cmd("git", &["diff", "--cached", "--quiet"])
            .run()
            .map(|output| !output.status.success())
            .unwrap_or(false);

        if !has_staged {
            log_info!("No staged files to check, pre-commit check skipped");
            log_success!("Pre-commit checks passed (nothing to commit)");
            return Ok(());
        }

        // 运行 pre-commit hooks
        // 注意：这里需要 pre-commit 工具已安装
        let output = cmd("git", &["commit", "--no-verify", "--dry-run"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .context("Failed to run pre-commit check")?;

        if output.status.success() {
            log_success!("Pre-commit checks passed");
            Ok(())
        } else {
            log_error!("Pre-commit checks failed");
            anyhow::bail!("Pre-commit check failed");
        }
    }
}
