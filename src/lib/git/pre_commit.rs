use anyhow::{Context, Result};
use duct::cmd;
use std::path::Path;
use std::process::Command;

use crate::{log_debug, log_info, log_success};

use super::commit::Git;

impl Git {
    /// 检查工程中是否存在 pre-commit hooks
    ///
    /// 检查以下位置：
    /// 1. `.git/hooks/pre-commit` - Git hooks
    /// 2. `.pre-commit-config.yaml` - pre-commit 工具配置文件
    /// 3. `pre-commit` 命令是否可用（pre-commit 工具）
    pub(crate) fn has_pre_commit() -> bool {
        // 检查 .git/hooks/pre-commit
        if Self::get_pre_commit_hook_path().is_some() {
            return true;
        }

        // 检查 .pre-commit-config.yaml
        if Path::new(".pre-commit-config.yaml").exists() {
            return true;
        }

        // 检查 pre-commit 命令是否可用
        if cmd("which", &["pre-commit"])
            .stdout_null()
            .stderr_null()
            .run()
            .is_ok()
        {
            return true;
        }

        false
    }

    /// 获取 pre-commit hook 路径（如果存在）
    fn get_pre_commit_hook_path() -> Option<std::path::PathBuf> {
        if let Ok(git_dir) = Git::get_git_dir() {
            let hooks_path = Path::new(&git_dir).join("hooks").join("pre-commit");
            if hooks_path.exists() && hooks_path.is_file() {
                return Some(hooks_path);
            }
        }
        None
    }

    /// 执行 pre-commit hooks（如果存在）
    ///
    /// 如果有 pre-commit 工具配置，使用 `pre-commit run`
    /// 如果有 Git hooks，直接执行 `.git/hooks/pre-commit` 脚本
    pub(crate) fn run_pre_commit() -> Result<()> {
        // 检查是否有 staged 的文件
        let has_staged = Self::has_staged().unwrap_or(false);

        if !has_staged {
            log_info!("No staged files, skipping pre-commit");
            return Ok(());
        }

        // 优先使用 pre-commit 工具
        if Path::new(".pre-commit-config.yaml").exists() {
            log_info!("Running pre-commit hooks...");
            let output = cmd("pre-commit", &["run"])
                .stdout_capture()
                .stderr_capture()
                .run()
                .context("Failed to run pre-commit")?;

            if output.status.success() {
                log_success!("Pre-commit checks passed");
                Ok(())
            } else {
                anyhow::bail!("Pre-commit checks failed");
            }
        } else if let Some(hooks_path) = Self::get_pre_commit_hook_path() {
            // 执行 Git hooks
            log_info!("Running Git pre-commit hooks...");
            let output = Command::new(&hooks_path)
                .output()
                .context("Failed to run pre-commit hooks")?;

            if output.status.success() {
                log_success!("Pre-commit checks passed");
                Ok(())
            } else {
                anyhow::bail!("Pre-commit checks failed");
            }
        } else {
            // 没有 pre-commit hooks，跳过
            log_debug!("No pre-commit hooks found, skipping");
            Ok(())
        }
    }
}
