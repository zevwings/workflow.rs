//! Commit amend command
//!
//! Amend the last commit, including message and files.
//! Provides interactive workflow following the implementation document.

use crate::base::dialog::{ConfirmDialog, InputDialog, MultiSelectDialog, SelectDialog};
use crate::commands::check;
use crate::git::{CommitInfo, GitBranch, GitCommit};
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

        // 步骤1: 检查是否有最后一次 commit
        if !GitCommit::has_last_commit()? {
            anyhow::bail!("❌ 错误：当前分支没有任何 commit\n\n无法执行 amend 操作，因为当前分支还没有任何提交记录。\n请先创建一个 commit。");
        }

        // 步骤2: 显示当前 commit 信息
        let commit_info = GitCommit::get_last_commit_info()?;
        let current_branch = GitBranch::current_branch()?;
        Self::display_commit_info(&commit_info, &current_branch)?;

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
                    let should_stage =
                        ConfirmDialog::new("工作区有未暂存的更改，是否暂存这些文件？")
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
        Self::preview_and_confirm(
            &commit_info,
            &new_message,
            &files_to_add,
            operation,
            &current_branch,
        )?;

        // 步骤6: 执行 amend
        // 6.1 暂存文件（如果需要）
        if !files_to_add.is_empty() {
            GitCommit::add_files(&files_to_add)?;
            for file in &files_to_add {
                log_success!("✓ 暂存文件: {}", file);
            }
        }

        // 6.2 和 6.3: 执行 amend（pre-commit hooks 在 amend 方法中处理）
        let new_sha = GitCommit::amend(new_message.as_deref(), no_edit, no_verify)?;

        log_break!();
        log_success!("✓ Commit amend 成功");
        log_info!("  新 Commit SHA: {}", &new_sha[..8]);
        if let Some(msg) = &new_message {
            log_info!("  提交消息: {}", msg);
        }

        // 6.4: 完成提示
        Self::display_completion_message(&current_branch, &commit_info.sha)?;

        Ok(())
    }

    /// 显示当前 commit 信息
    fn display_commit_info(commit_info: &CommitInfo, branch: &str) -> Result<()> {
        log_break!();
        log_message!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        );
        log_message!("                           当前 Commit 信息");
        log_message!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        );
        log_message!("");
        log_info!("  Commit SHA:    {}", &commit_info.sha[..8]);
        log_info!("  提交消息:      {}", commit_info.message);
        log_info!("  作者:          {}", commit_info.author);
        log_info!("  日期:          {}", commit_info.date);
        log_info!("  分支:          {}", branch);
        log_message!("");

        // 显示工作区状态
        let status = GitCommit::status()?;
        let modified_count =
            status.lines().filter(|l| l.starts_with(" M") || l.starts_with("M ")).count();
        let staged_count = status
            .lines()
            .filter(|l| l.starts_with("M ") || l.starts_with("A ") || l.starts_with("D "))
            .count();
        let untracked_count = status.lines().filter(|l| l.starts_with("??")).count();

        log_info!("  工作区状态:");
        log_info!("    - 已修改文件:  {} 个", modified_count);
        log_info!("    - 已暂存文件:  {} 个", staged_count);
        log_info!("    - 未跟踪文件:  {} 个", untracked_count);
        log_message!("");
        log_message!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        );

        Ok(())
    }

    /// 选择操作类型
    fn select_operation() -> Result<AmendOperation> {
        let options = vec![
            "仅修改提交消息",
            "仅添加文件",
            "修改消息并添加文件",
            "不修改，仅重新提交",
        ];

        let selected = SelectDialog::new("请选择要执行的操作", options)
            .prompt()
            .context("Failed to select operation")?;

        match selected {
            "仅修改提交消息" => Ok(AmendOperation::MessageOnly),
            "仅添加文件" => Ok(AmendOperation::FilesOnly),
            "修改消息并添加文件" => Ok(AmendOperation::MessageAndFiles),
            "不修改，仅重新提交" => Ok(AmendOperation::NoEdit),
            _ => unreachable!(),
        }
    }

    /// 输入新提交消息
    fn input_new_message(current_message: &str) -> Result<String> {
        let new_message = InputDialog::new("请输入新的提交消息")
            .with_default(current_message)
            .with_validator(|msg: &str| {
                if msg.trim().is_empty() {
                    Err("提交消息不能为空".to_string())
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
            log_warning!("没有可添加的文件");
            return Ok(Vec::new());
        }

        let selected_files = MultiSelectDialog::new("请选择要添加到 commit 的文件", all_files)
            .prompt()
            .context("Failed to select files")?;

        if selected_files.is_empty() {
            // 询问是否继续（仅修改消息）
            let continue_without_files = ConfirmDialog::new("没有选择文件，是否仅修改提交消息？")
                .with_default(true)
                .prompt()?;

            if !continue_without_files {
                anyhow::bail!("操作已取消");
            }
        }

        Ok(selected_files)
    }

    /// 预览和确认
    fn preview_and_confirm(
        commit_info: &CommitInfo,
        new_message: &Option<String>,
        files_to_add: &[String],
        operation: AmendOperation,
        current_branch: &str,
    ) -> Result<()> {
        log_break!();
        log_message!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        );
        log_message!("                           Commit Amend 预览");
        log_message!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        );
        log_message!("");
        log_info!("  原 Commit SHA:  {}", &commit_info.sha[..8]);
        log_info!("  新 Commit SHA:  (将重新生成)");
        log_message!("");

        if let Some(msg) = new_message {
            log_info!("  原提交消息:     {}", commit_info.message);
            log_info!("  新提交消息:     {}", msg);
        } else {
            log_info!("  提交消息:       {} (不修改)", commit_info.message);
        }
        log_message!("");

        if !files_to_add.is_empty() {
            log_info!("  要添加的文件:");
            for file in files_to_add {
                log_info!("    ✓ {}", file);
            }
        } else {
            log_info!("  要添加的文件:   无");
        }
        log_message!("");

        let operation_str = match operation {
            AmendOperation::MessageOnly => "仅修改提交消息",
            AmendOperation::FilesOnly => "仅添加文件",
            AmendOperation::MessageAndFiles => "修改消息并添加文件",
            AmendOperation::NoEdit => "不修改，仅重新提交",
        };
        log_info!("  操作类型:       {}", operation_str);
        log_message!("");
        log_message!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        );

        // 检查 commit 是否已推送
        let is_pushed =
            GitBranch::is_commit_in_remote(current_branch, &commit_info.sha).unwrap_or(false);

        if is_pushed {
            log_warning!("⚠️  警告：此 commit 可能已经推送到远程仓库");
            log_message!("");
            log_info!("执行 amend 后，需要使用 force push 更新远程分支：");
            log_info!("  git push --force");
            log_message!("");
            log_info!("这可能会影响其他协作者，请确保已通知团队成员。");
            log_message!("");
        }

        // 最终确认
        ConfirmDialog::new("确认执行 commit amend？")
            .with_default(true)
            .with_cancel_message("操作已取消")
            .prompt()
            .context("Failed to get confirmation")?;

        Ok(())
    }

    /// 显示完成提示
    fn display_completion_message(current_branch: &str, old_sha: &str) -> Result<()> {
        // 检查 commit 是否已推送
        let is_pushed = GitBranch::is_commit_in_remote(current_branch, old_sha).unwrap_or(false);

        if is_pushed {
            log_break!();
            log_message!(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            );
            log_message!("                          Commit Amend 完成");
            log_message!(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            );
            log_message!("");
            log_success!("  ✓ Commit 已修改");
            log_message!("");
            log_info!("  提示：");
            log_info!("    - 如果此 commit 已推送到远程，需要使用 force push 更新：");
            log_info!("      git push --force");
            log_message!("");
            log_message!(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            );
        }

        Ok(())
    }
}
