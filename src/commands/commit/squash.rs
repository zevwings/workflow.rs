//! Commit squash command
//!
//! Squash multiple commits into one, simplifying commit history.
//! Provides interactive workflow with multi-select.

use crate::commands::check;
use crate::commands::commit::helpers::{
    check_has_last_commit, check_not_on_default_branch, handle_force_push_warning,
};
use crate::commit::{CommitSquash, SquashOptions};
use crate::{br, info, success};
use color_eyre::{eyre::WrapErr, Result};

/// Commit squash command
pub struct CommitSquashCommand;

impl CommitSquashCommand {
    /// Execute the commit squash command
    pub fn execute() -> Result<()> {
        // 0. 检查并确保仓库配置存在
        crate::commands::repo::setup::RepoSetupCommand::ensure()?;

        // 1. Run checks
        check::CheckCommand::run_all()?;

        br!();
        info!("Commit Squash");

        // 步骤0: 检查是否是默认分支（保护分支不允许直接修改提交历史）
        let (current_branch, _default_branch) = check_not_on_default_branch("squash")?;

        // 步骤1: 检查是否有最后一次 commit
        check_has_last_commit()?;

        // 步骤2: 获取当前分支创建之后的提交
        br!();
        info!(
            "Getting commits created after branch '{}' was created...",
            current_branch
        );

        let commits = CommitSquash::get_branch_commits(&current_branch)
            .wrap_err("Failed to get branch commits")?;

        if commits.is_empty() {
            color_eyre::eyre::bail!(
                "❌ Error: No commits found after branch '{}' was created\n\nCannot squash commits because there are no commits unique to this branch.",
                current_branch
            );
        }

        // 步骤3: 显示可用的 commits 并让用户多选
        br!();
        info!(
            "Found {} commit(s) created after branch was created:",
            commits.len()
        );
        for (idx, commit) in commits.iter().enumerate() {
            let marker = if idx == 0 { "[OLDEST] " } else { "" };
            info!(
                "  {}. {}[{}] {}",
                idx + 1,
                marker,
                &commit.sha[..8],
                commit.message
            );
        }

        // 步骤4: 使用 MultiSelectDialog 让用户选择要压缩的 commits
        let options: Vec<String> = commits
            .iter()
            .enumerate()
            .map(|(idx, c)| {
                let marker = if idx == 0 { "[OLDEST] " } else { "" };
                format!("{}[{}] {}", marker, &c.sha[..8], c.message)
            })
            .collect();

        let selected_options = crate::multiselect!(
            "Select commits to squash (use space to select, enter to confirm)",
            options
        )
        .prompt()
        .wrap_err("Failed to select commits")?;

        if selected_options.is_empty() {
            color_eyre::eyre::bail!("No commits selected. Operation cancelled.");
        }

        // 步骤5: 从选中的选项解析出 commit SHA
        let selected_commits: Vec<_> = selected_options
            .iter()
            .filter_map(|option| {
                // 格式: "[OLDEST] [abc1234] message" 或 "[def5678] message"
                let sha_start = if let Some(after_oldest) = option.strip_prefix("[OLDEST] [") {
                    // 格式: "[OLDEST] [abc1234] message"
                    after_oldest.find(']').map(|end_pos| after_oldest[..end_pos].to_string())
                } else if option.starts_with('[') {
                    // 格式: "[def5678] message"
                    let sha_start = option.find('[')? + 1;
                    let sha_part = &option[sha_start..];
                    sha_part.find(']').map(|end_pos| sha_part[..end_pos].to_string())
                } else {
                    None
                };

                sha_start.and_then(|sha| commits.iter().find(|c| c.sha.starts_with(&sha)).cloned())
            })
            .collect();

        if selected_commits.is_empty() {
            color_eyre::eyre::bail!("Failed to parse selected commits");
        }

        // 确保选中的 commits 按时间顺序排列（从旧到新）
        let mut selected_commits_sorted = selected_commits;
        selected_commits_sorted.sort_by(|a, b| {
            commits
                .iter()
                .position(|c| c.sha == a.sha)
                .cmp(&commits.iter().position(|c| c.sha == b.sha))
        });

        br!();
        info!(
            "Selected {} commit(s) to squash:",
            selected_commits_sorted.len()
        );
        for (idx, commit) in selected_commits_sorted.iter().enumerate() {
            info!("  {}. [{}] {}", idx + 1, &commit.sha[..8], commit.message);
        }

        // 步骤6: 输入新的提交消息
        br!();
        let new_message = crate::input!("Enter new commit message for squashed commit")
            .validator(|msg: &str| {
                if msg.trim().is_empty() {
                    Err("Commit message cannot be empty".to_string())
                } else {
                    Ok(())
                }
            })
            .prompt()
            .wrap_err("Failed to get new commit message")?;

        // 步骤7: 预览和确认
        let preview =
            CommitSquash::create_preview(&selected_commits_sorted, &new_message, &current_branch)
                .wrap_err("Failed to create preview")?;

        br!();
        info!("{}", CommitSquash::format_preview(&preview));
        info!("");

        // 最终确认
        crate::confirm!("Confirm to execute commit squash?")
            .default(true)
            .prompt()
            .wrap_err("Failed to get confirmation")?;

        // 步骤8: 执行 squash
        let commit_shas: Vec<String> =
            selected_commits_sorted.iter().map(|c| c.sha.clone()).collect();

        let options = SquashOptions {
            commit_shas: commit_shas.clone(),
            new_message: new_message.clone(),
            auto_stash: true,
        };

        CommitSquash::execute_squash(options).wrap_err("Failed to execute squash")?;

        br!();
        success!("✓ Commit squash successful");
        info!(
            "  Squashed {} commit(s) into one",
            selected_commits_sorted.len()
        );
        info!("  New commit message: {}", new_message);

        // 步骤9: 显示完成提示
        if let Some(msg) = CommitSquash::format_completion_message(&current_branch, &commit_shas)? {
            br!();
            info!("{}", msg);
        }

        // 步骤10: 如果 commits 已推送，询问是否要 force push
        handle_force_push_warning(
            &current_branch,
            &commit_shas[0], // 使用第一个 commit 的 SHA 来检查
            |branch, _sha| CommitSquash::should_show_force_push_warning(branch, &commit_shas),
        )?;

        Ok(())
    }
}
