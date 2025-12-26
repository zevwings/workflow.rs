//! Commit 命令公共帮助函数
//!
//! 提供 Commit 命令之间共享的公共功能，避免代码重复。

use crate::base::dialog::ConfirmDialog;
use crate::git::{GitBranch, GitCommit};
use crate::{log_break, log_info, log_success};
use color_eyre::{eyre::WrapErr, Result};

/// 检查是否在默认分支上（保护分支不允许修改提交历史）
///
/// # 参数
///
/// * `operation_name` - 操作名称（用于错误消息，如 "amend" 或 "reword"）
///
/// # 返回
///
/// 如果在默认分支上，返回错误；否则返回当前分支名和默认分支名
pub fn check_not_on_default_branch(operation_name: &str) -> Result<(String, String)> {
    check_not_on_default_branch_in(
        std::env::current_dir().wrap_err("Failed to get current directory")?,
        operation_name,
    )
}

/// 检查是否在默认分支上（保护分支不允许修改提交历史，指定仓库路径）
///
/// # 参数
///
/// * `repo_path` - 仓库根目录路径
/// * `operation_name` - 操作名称（用于错误消息，如 "amend" 或 "reword"）
///
/// # 返回
///
/// 如果在默认分支上，返回错误；否则返回当前分支名和默认分支名
pub fn check_not_on_default_branch_in(
    repo_path: impl AsRef<std::path::Path>,
    operation_name: &str,
) -> Result<(String, String)> {
    let current_branch = GitBranch::current_branch_in(repo_path.as_ref())
        .wrap_err("Failed to get current branch")?;
    let default_branch = GitBranch::get_default_branch_in(repo_path.as_ref())
        .wrap_err("Failed to get default branch")?;

    if current_branch == default_branch {
        color_eyre::eyre::bail!(
            "❌ Error: Cannot {} commits on protected branch '{}'\n\nProtected branches (default branches) do not allow direct modification of commit history.\nPlease switch to a feature branch first.",
            operation_name,
            default_branch
        );
    }

    Ok((current_branch, default_branch))
}

/// 处理 force push 警告和确认
///
/// 如果 commit 已推送，询问用户是否要 force push。
///
/// # 参数
///
/// * `current_branch` - 当前分支名称
/// * `commit_sha` - Commit SHA
/// * `should_show_warning` - 是否应该显示警告的函数（如 `CommitAmend::should_show_force_push_warning`）
///
/// # 返回
///
/// 如果用户选择推送，返回 `Ok(())`；如果用户取消，也返回 `Ok(())`（不视为错误）
pub fn handle_force_push_warning<F>(
    current_branch: &str,
    commit_sha: &str,
    should_show_warning: F,
) -> Result<()>
where
    F: FnOnce(&str, &str) -> Result<bool>,
{
    let is_pushed = should_show_warning(current_branch, commit_sha)?;

    if is_pushed {
        log_break!();
        let should_push = ConfirmDialog::new("Push to remote (force-with-lease)?")
            .with_default(true)
            .with_cancel_message("Push cancelled by user")
            .prompt()
            .wrap_err("Failed to get push confirmation")?;

        if should_push {
            log_break!();
            log_info!("Pushing to remote (force-with-lease)...");
            log_break!();
            GitBranch::push_force_with_lease(current_branch)
                .wrap_err("Failed to push to remote (force-with-lease)")?;
            log_break!();
            log_success!("Pushed to remote successfully");
        } else {
            log_info!("Skipping push as requested by user");
            log_info!("You can push manually with: git push --force-with-lease");
        }
    }

    Ok(())
}

/// 检查是否有最后一次 commit
///
/// # 返回
///
/// 如果没有 commit，返回错误；否则返回 `Ok(())`
pub fn check_has_last_commit() -> Result<()> {
    check_has_last_commit_in(std::env::current_dir().wrap_err("Failed to get current directory")?)
}

/// 检查是否有最后一次 commit（指定仓库路径）
///
/// # 参数
///
/// * `repo_path` - 仓库根目录路径
///
/// # 返回
///
/// 如果没有 commit，返回错误；否则返回 `Ok(())`
pub fn check_has_last_commit_in(repo_path: impl AsRef<std::path::Path>) -> Result<()> {
    if !GitCommit::has_last_commit_in(repo_path)? {
        color_eyre::eyre::bail!(
            "❌ Error: No commits found in current branch\n\nCannot perform operation because the current branch has no commit history.\nPlease create a commit first."
        );
    }
    Ok(())
}
