use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use duct::cmd;

use super::commit::GitCommit;
use super::repo::GitRepo;
use crate::{log_break, log_info, log_success};

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
            // 检查是否是我们的标准 pre-commit hook（包含代码质量检查）
            if Self::is_standard_pre_commit_hook(&hooks_path) {
                // 使用 Rust 实现代码质量检查，统一使用日志宏
                Self::run_code_quality_checks()?;
                Ok(PreCommitResult {
                    executed: true,
                    messages: vec!["Code quality checks passed".to_string()],
                })
            } else {
                // 执行其他 Git hooks
                let output = Command::new(&hooks_path)
                    .output()
                    .context("Failed to run pre-commit hooks")?;

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

    /// Check if this is a standard pre-commit hook (contains code quality checks)
    fn is_standard_pre_commit_hook(hooks_path: &std::path::Path) -> bool {
        // Check if file content contains our standard check logic
        if let Ok(content) = std::fs::read_to_string(hooks_path) {
            content.contains("运行代码质量检查")
                || content.contains("Code quality check")
                || content.contains("Running code quality checks")
        } else {
            false
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

    /// Run code quality checks (using log macros for unified output)
    ///
    /// Includes:
    /// 1. Auto-format code
    /// 2. Update staged files
    /// 3. Run full code checks (format, Clippy, Check)
    fn run_code_quality_checks() -> Result<()> {
        log_break!('=', 38);
        log_info!("🔍 Running code quality checks (Pre-commit Hook)");
        log_break!('=', 38);
        log_break!();

        // 1. Auto-format code
        log_info!("📝 Auto-formatting code...");
        let fmt_output = cmd("cargo", &["fmt"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .context("Failed to run cargo fmt")?;

        if !fmt_output.status.success() {
            anyhow::bail!("Code formatting failed");
        }
        log_success!("Code formatting completed");
        log_break!();

        // 2. Add formatted files to staging area
        log_info!("📦 Updating staged files...");
        GitCommit::add_all().context("Failed to update staged files")?;
        log_success!("Staged files updated");
        log_break!();

        // 3. Run full code checks
        log_info!("Running full code checks...");
        log_break!();

        // 3.1 Check code format
        log_info!("1/3 Checking code format...");
        let fmt_check = cmd("cargo", &["fmt", "--check"])
            .stdout_capture()
            .stderr_capture()
            .run();

        match fmt_check {
            Ok(output) if output.status.success() => {
                log_success!("Code format is correct");
            }
            _ => {
                anyhow::bail!("Code format is incorrect, run 'cargo fmt' to auto-fix");
            }
        }
        log_break!();

        // 3.2 Run Clippy check
        log_info!("2/3 Running Clippy check...");
        let clippy_output = cmd("cargo", &["clippy", "--", "-D", "warnings"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .context("Failed to run cargo clippy")?;

        if !clippy_output.status.success() {
            anyhow::bail!("Clippy check failed");
        }
        log_success!("Clippy check passed");
        log_break!();

        // 3.3 Run cargo check
        log_info!("3/3 Running cargo check...");
        let check_output = cmd("cargo", &["check"])
            .stdout_capture()
            .stderr_capture()
            .run()
            .context("Failed to run cargo check")?;

        if !check_output.status.success() {
            anyhow::bail!("Cargo check failed");
        }
        log_success!("Check passed");
        log_break!();

        log_success!("All checks passed!");
        log_break!();
        log_success!("✅ All checks passed, continuing with commit...");
        log_break!();

        Ok(())
    }
}
