//! Commit amend command
//!
//! Amend the last commit, including message and files.
//! Provides interactive workflow following the implementation document.

use crate::base::dialog::{ConfirmDialog, InputDialog, MultiSelectDialog, SelectDialog};
use crate::commands::check;
use crate::commit::CommitAmend;
use crate::git::{GitBranch, GitCommit};
use crate::{log_break, log_info, log_message, log_success, log_warning};
use anyhow::{Context, Result};

/// Commit amend command
pub struct CommitAmendCommand;

/// 操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AmendOperation {
    /// 仅修改提交消息
    MessageOnly,
    /// 仅添加文件
    FilesOnly,
    /// 修改消息并添加文件
    MessageAndFiles,
    /// 不修改，仅重新提交
    NoEdit,
}

impl CommitAmendCommand {
    /// Execute the commit amend command
    ///
    /// # Arguments
    ///
    /// * `message` - New commit message (optional, will prompt if not provided)
    /// * `no_edit` - Don't edit the commit message
    /// * `no_verify` - Skip pre-commit hooks
    pub fn execute(message: Option<String>, no_edit: bool, no_verify: bool) -> Result<()> {
        // 1. Run checks
        check::CheckCommand::run_all()?;

        log_break!();
        log_message!("Commit Amend");

        // 步骤0: 检查是否是默认分支（保护分支不允许直接修改提交历史）
        let current_branch = GitBranch::current_branch().context("Failed to get current branch")?;
        let default_branch =
            GitBranch::get_default_branch().context("Failed to get default branch")?;
        if current_branch == default_branch {
            anyhow::bail!(
                "❌ Error: Cannot amend commits on protected branch '{}'\n\nProtected branches (default branches) do not allow direct modification of commit history.\nPlease switch to a feature branch first.",
                default_branch
            );
        }

        // 步骤1: 检查是否有最后一次 commit
        if !GitCommit::has_last_commit()? {
            anyhow::bail!("❌ Error: No commits found in current branch\n\nCannot perform amend operation because the current branch has no commit history.\nPlease create a commit first.");
        }

        // 步骤2: 显示当前 commit 信息
        let commit_info = GitCommit::get_last_commit_info()?;
        let status = GitCommit::get_worktree_status()?;
        log_break!();
        log_message!(
            "{}",
            CommitAmend::format_commit_info_detailed(&commit_info, &current_branch, Some(&status))
        );

        // 步骤3: 选择操作类型（如果提供了参数，跳过交互）
        let operation = if no_edit {
            AmendOperation::NoEdit
        } else if message.is_some() {
            // 如果提供了消息，默认是仅修改消息
            AmendOperation::MessageOnly
        } else {
            // 交互式选择
            Self::select_operation()?
        };

        // 步骤4: 根据选择执行操作
        let (new_message, files_to_add) = match operation {
            AmendOperation::MessageOnly => {
                let msg = if let Some(m) = message {
                    m
                } else {
                    Self::input_new_message(&commit_info.message)?
                };
                (Some(msg), Vec::new())
            }
            AmendOperation::FilesOnly => {
                let files = Self::select_files_to_add()?;
                (None, files)
            }
            AmendOperation::MessageAndFiles => {
                let msg = Self::input_new_message(&commit_info.message)?;
                let files = Self::select_files_to_add()?;
                (Some(msg), files)
            }
            AmendOperation::NoEdit => {
                // 检查是否有未暂存的更改
                if GitCommit::has_commit()? {
                    let should_stage = ConfirmDialog::new(
                        "Working directory has unstaged changes. Stage these files?",
                    )
                    .with_default(true)
                    .prompt()?;
                    if should_stage {
                        GitCommit::add_all()?;
                    }
                }
                (None, Vec::new())
            }
        };

        // 步骤5: 预览和确认
        let operation_str = match operation {
            AmendOperation::MessageOnly => "Modify message only",
            AmendOperation::FilesOnly => "Add files only",
            AmendOperation::MessageAndFiles => "Modify message and add files",
            AmendOperation::NoEdit => "No changes, just recommit",
        };
        let preview = CommitAmend::create_preview(
            &commit_info,
            &new_message,
            &files_to_add,
            operation_str,
            &current_branch,
        )?;
        log_break!();
        log_message!("{}", CommitAmend::format_preview(&preview));
        log_message!("");

        // 最终确认
        ConfirmDialog::new("Confirm to execute commit amend?")
            .with_default(true)
            .with_cancel_message("Operation cancelled")
            .prompt()
            .context("Failed to get confirmation")?;

        // 步骤6: 执行 amend
        // 6.1 暂存文件（如果需要）
        if !files_to_add.is_empty() {
            GitCommit::add_files(&files_to_add)?;
            for file in &files_to_add {
                log_success!("✓ Staged file: {}", file);
            }
        }

        // 6.2 和 6.3: 执行 amend（pre-commit hooks 在 amend 方法中处理）
        let new_sha = GitCommit::amend(new_message.as_deref(), no_edit, no_verify)?;

        log_break!();
        log_success!("✓ Commit amend successful");
        log_info!("  New Commit SHA: {}", &new_sha[..8]);
        if let Some(msg) = &new_message {
            log_info!("  Commit message: {}", msg);
        }

        // 6.4: 完成提示
        if let Some(msg) =
            CommitAmend::format_completion_message(&current_branch, &commit_info.sha)?
        {
            log_break!();
            log_message!("{}", msg);
        }

        Ok(())
    }

    /// 选择操作类型
    fn select_operation() -> Result<AmendOperation> {
        let options = vec![
            "Modify message only",
            "Add files only",
            "Modify message and add files",
            "No changes, just recommit",
        ];

        let selected = SelectDialog::new("Select operation to perform", options)
            .prompt()
            .context("Failed to select operation")?;

        match selected {
            "Modify message only" => Ok(AmendOperation::MessageOnly),
            "Add files only" => Ok(AmendOperation::FilesOnly),
            "Modify message and add files" => Ok(AmendOperation::MessageAndFiles),
            "No changes, just recommit" => Ok(AmendOperation::NoEdit),
            _ => unreachable!(),
        }
    }

    /// 输入新提交消息
    fn input_new_message(current_message: &str) -> Result<String> {
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

        Ok(new_message)
    }

    /// 选择要添加的文件
    fn select_files_to_add() -> Result<Vec<String>> {
        // 获取修改的文件
        let modified_files = GitCommit::get_modified_files()?;
        let untracked_files = GitCommit::get_untracked_files()?;

        let all_files: Vec<String> = modified_files.into_iter().chain(untracked_files).collect();

        if all_files.is_empty() {
            log_warning!("No files available to add");
            return Ok(Vec::new());
        }

        let selected_files = MultiSelectDialog::new("Select files to add to commit", all_files)
            .prompt()
            .context("Failed to select files")?;

        if selected_files.is_empty() {
            // 询问是否继续（仅修改消息）
            let continue_without_files =
                ConfirmDialog::new("No files selected. Modify message only?")
                    .with_default(true)
                    .prompt()?;

            if !continue_without_files {
                anyhow::bail!("Operation cancelled");
            }
        }

        Ok(selected_files)
    }
}
