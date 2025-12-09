use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use duct::cmd;

use super::commit::GitCommit;
use super::repo::GitRepo;

/// Pre-commit 执行结果
#[derive(Debug, Clone)]
#[allow(dead_code)] // 字段将在调用者更新后使用
pub struct PreCommitResult {
    /// 是否执行了 pre-commit
    pub executed: bool,
    /// 消息
    pub messages: Vec<String>,
}

/// Git Pre-commit Hooks 管理
///
/// 提供 pre-commit hooks 相关的操作功能，包括：
/// - 检查是否存在 pre-commit hooks
/// - 执行 pre-commit hooks
pub struct GitPreCommit;

impl GitPreCommit {
    /// Check if pre-commit hooks exist in the project
    ///
    /// Checks the following locations:
    /// 1. `.git/hooks/pre-commit` - Git hooks
    /// 2. `.pre-commit-config.yaml` - pre-commit tool config file
    /// 3. Whether `pre-commit` command is available (pre-commit tool)
    pub fn has_pre_commit() -> bool {
        // 检查 .git/hooks/pre-commit
        if Self::get_pre_commit_hook_path().is_some() {
            return true;
        }

        // 检查 .pre-commit-config.yaml
        if Path::new(".pre-commit-config.yaml").exists() {
            return true;
        }

        // 检查 pre-commit 命令是否可用
        if cmd("which", &["pre-commit"]).stdout_null().stderr_null().run().is_ok() {
            return true;
        }

        false
    }

    /// 获取 pre-commit hook 路径（如果存在）
    fn get_pre_commit_hook_path() -> Option<std::path::PathBuf> {
        if let Ok(git_dir) = GitRepo::get_git_dir() {
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
    pub(crate) fn run_pre_commit() -> Result<PreCommitResult> {
        // 检查是否有 staged 的文件
        let has_staged = GitCommit::has_staged().unwrap_or(false);

        if !has_staged {
            return Ok(PreCommitResult {
                executed: false,
                messages: vec!["No staged files, skipping pre-commit".to_string()],
            });
        }

        // 优先使用 pre-commit 工具
        if Path::new(".pre-commit-config.yaml").exists() {
            let output = cmd("pre-commit", &["run"])
                .stdout_capture()
                .stderr_capture()
                .run()
                .context("Failed to run pre-commit")?;

            if output.status.success() {
                Ok(PreCommitResult {
                    executed: true,
                    messages: vec![
                        "Running pre-commit hooks...".to_string(),
                        "Pre-commit checks passed".to_string(),
                    ],
                })
            } else {
                anyhow::bail!("Pre-commit checks failed");
            }
        } else if let Some(hooks_path) = Self::get_pre_commit_hook_path() {
            // 执行 Git pre-commit hook 脚本
            let output =
                Command::new(&hooks_path).output().context("Failed to run pre-commit hooks")?;

            if output.status.success() {
                Ok(PreCommitResult {
                    executed: true,
                    messages: vec![
                        "Running Git pre-commit hooks...".to_string(),
                        "Pre-commit checks passed".to_string(),
                    ],
                })
            } else {
                anyhow::bail!("Pre-commit checks failed");
            }
        } else {
            // 没有 pre-commit hooks，跳过
            crate::trace_debug!("No pre-commit hooks found, skipping");
            Ok(PreCommitResult {
                executed: false,
                messages: vec![],
            })
        }
    }

    /// Public method to run pre-commit checks (for use outside of commit flow)
    ///
    /// This method should be called before committing to run pre-commit checks
    /// without interference from Spinner output. It will:
    /// 1. Check if pre-commit hooks exist
    /// 2. Run the checks if they exist
    /// 3. Return an error if checks fail
    pub fn run_checks() -> Result<()> {
        if Self::has_pre_commit() {
            // First, stage all files (needed for pre-commit checks)
            GitCommit::add_all().context("Failed to stage files for pre-commit checks")?;
            Self::run_pre_commit()?;
        }
        Ok(())
    }
}
