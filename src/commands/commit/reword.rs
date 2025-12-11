//! Commit reword command
//!
//! Reword a commit message without changing its content.
//! Provides interactive workflow following the implementation document.

use crate::base::dialog::{ConfirmDialog, InputDialog, SelectDialog};
use crate::commands::check;
use crate::commands::commit::helpers::{check_has_last_commit, check_not_on_default_branch, handle_force_push_warning};
use crate::commit::{CommitReword, RewordHistoryOptions};
use crate::git::{CommitInfo, GitCommit};
use crate::{log_break, log_info, log_message, log_success};
use anyhow::{Context, Result};

/// Commit reword command
pub struct CommitRewordCommand;

impl CommitRewordCommand {
    /// Execute the commit reword command
    ///
    /// # Arguments
    ///
    /// * `commit_id` - Optional commit reference (defaults to HEAD if not provided)
    pub fn execute(commit_id: Option<String>) -> Result<()> {
        // 1. Run checks
        check::CheckCommand::run_all()?;

        log_break!();
        log_message!("Commit Reword");

        // 步骤0: 检查是否是默认分支（保护分支不允许直接修改提交历史）
        let (current_branch, _default_branch) = check_not_on_default_branch("reword")?;

        // 步骤1: 检查是否有最后一次 commit
        check_has_last_commit()?;

        // 步骤2: 解析 commit 引用（无参数时默认 HEAD）
        let commit_ref = commit_id.as_deref().unwrap_or("HEAD");
        let commit_sha = GitCommit::parse_commit_ref(commit_ref)
            .with_context(|| format!("Failed to parse commit reference: {}", commit_ref))?;

        // 步骤3: 验证 commit 是否在当前分支历史中
        if !GitCommit::is_commit_in_current_branch(&commit_sha)? {
            anyhow::bail!(
                "❌ Error: Commit {} is not in the current branch history\n\nYou can only reword commits from the current branch.",
                &commit_sha[..8]
            );
        }

        // 步骤4: 获取 commit 信息
        let mut commit_info = GitCommit::get_commit_info(&commit_sha)
            .with_context(|| format!("Failed to get commit info: {}", &commit_sha[..8]))?;

        // 步骤5: 显示 commit 信息并确认
        log_break!();
        log_message!(
            "{}",
            CommitReword::format_commit_info(&commit_info, &current_branch)
        );

        // 步骤6: 确认使用此 commit
        if !ConfirmDialog::new("Confirm to reword this commit?")
            .with_default(true)
            .prompt()
            .context("Failed to get user confirmation")?
        {
            // 用户选择重新选择
            commit_info = Self::select_commit_interactively()?;
        }

        // 步骤7: 输入新消息并确认
        let new_message = Self::input_new_message_with_confirm(&commit_info.message)?;

        // 步骤8: 检查是否是 HEAD
        let is_head = GitCommit::is_head_commit(&commit_info.sha)?;

        // 步骤9: 预览和确认
        let preview =
            CommitReword::create_preview(&commit_info, &new_message, is_head, &current_branch)?;
        log_break!();
        log_message!("{}", CommitReword::format_preview(&preview));
        log_message!("");

        // 最终确认
        ConfirmDialog::new("Confirm to execute commit reword?")
            .with_default(true)
            .with_cancel_message("Operation cancelled")
            .prompt()
            .context("Failed to get confirmation")?;

        // 步骤10: 执行修改
        if is_head {
            // 修改 HEAD，使用 amend
            let new_sha = GitCommit::amend(Some(&new_message), false, false)?;
            log_break!();
            log_success!("✓ Commit reword successful");
            log_info!("  New Commit SHA: {}", &new_sha[..8]);
            log_info!("  New commit message: {}", new_message);
        } else {
            // 修改历史 commit，使用 rebase -i
            let options = RewordHistoryOptions {
                commit_sha: commit_info.sha.clone(),
                new_message: new_message.clone(),
                auto_stash: true,
            };
            CommitReword::reword_history_commit(options)?;
            log_break!();
            log_success!("✓ Commit reword successful");
            log_info!("  New commit message: {}", new_message);
        }

        // 步骤11: 显示完成提示
        if let Some(msg) =
            CommitReword::format_completion_message(&current_branch, &commit_info.sha)?
        {
            log_break!();
            log_message!("{}", msg);
        }

        // 步骤12: 如果 commit 已推送，询问是否要 force push
        handle_force_push_warning(
            &current_branch,
            &commit_info.sha,
            |branch, sha| CommitReword::should_show_force_push_warning(branch, sha),
        )?;

        Ok(())
    }

    /// 交互式选择 commit（支持 fuzzy-matcher）
    fn select_commit_interactively() -> Result<CommitInfo> {
        let commits = GitCommit::get_branch_commits(20)?;

        if commits.is_empty() {
            anyhow::bail!("No commits available");
        }

        // 格式化选项：显示 SHA、消息等信息
        let options: Vec<String> = commits
            .iter()
            .enumerate()
            .map(|(idx, c)| {
                // 标记 HEAD（第一个）
                let marker = if idx == 0 { "[HEAD] " } else { "" };
                format!("{}[{}] {}", marker, &c.sha[..8], c.message)
            })
            .collect();

        // SelectDialog 默认支持 fuzzy-matcher
        let selected = SelectDialog::new("Select commit to reword", options)
            .with_default(0) // 默认选中 HEAD
            .prompt()
            .context("Failed to select commit")?;

        // 从选中的选项解析出 commit SHA
        // 格式: "[HEAD] [abc1234] message" 或 "[def5678] message"
        let commit_sha = if selected.starts_with("[HEAD] [") {
            // 格式: "[HEAD] [abc1234] message"
            let sha_start = selected.find('[').and_then(|pos1| {
                let after_first = &selected[pos1 + 1..];
                if after_first.starts_with("HEAD] [") {
                    let sha_start_pos = after_first.find('[').map(|pos2| pos2 + 1)?;
                    let sha_part = &after_first[sha_start_pos..];
                    sha_part.find(']').map(|end_pos| sha_part[..end_pos].to_string())
                } else {
                    None
                }
            });
            sha_start.ok_or_else(|| anyhow::anyhow!("Failed to parse selected commit"))?
        } else if selected.starts_with('[') {
            // 格式: "[def5678] message"
            let sha_start = selected
                .find('[')
                .ok_or_else(|| anyhow::anyhow!("Failed to parse selected commit"))?
                + 1;
            let sha_part = &selected[sha_start..];
            sha_part
                .find(']')
                .map(|end_pos| sha_part[..end_pos].to_string())
                .ok_or_else(|| anyhow::anyhow!("Failed to parse selected commit"))?
        } else {
            anyhow::bail!("Failed to parse selected commit format");
        };

        // 从 commits 列表中找到对应的 CommitInfo
        commits
            .into_iter()
            .find(|c| c.sha.starts_with(&commit_sha))
            .ok_or_else(|| anyhow::anyhow!("Selected commit not found"))
    }

    /// 输入新消息并确认（支持重新输入）
    fn input_new_message_with_confirm(current_message: &str) -> Result<String> {
        loop {
            // 输入新消息
            let new_message = InputDialog::new("Enter new commit message")
                .with_default(current_message)
                .with_validator(|msg: &str| {
                    if msg.trim().is_empty() {
                        Err("Commit message cannot be empty".to_string())
                    } else {
                        Ok(())
                    }
                })
                .prompt()
                .context("Failed to get new commit message")?;

            // 确认修改
            if ConfirmDialog::new("Confirm to reword this commit?")
                .with_default(true)
                .prompt()
                .context("Failed to get confirmation")?
            {
                return Ok(new_message);
            } else {
                // 询问是否重新输入
                if !ConfirmDialog::new("Re-enter message?")
                    .with_default(true)
                    .prompt()
                    .context("Failed to get user choice")?
                {
                    anyhow::bail!("Operation cancelled");
                }
                // 继续循环，重新输入
            }
        }
    }
}
