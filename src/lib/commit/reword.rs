//! Commit Reword 业务逻辑
//!
//! 提供 reword 操作相关的业务逻辑，包括：
//! - 预览信息生成
//! - 格式化显示
//! - 历史 commit reword 执行
//! - Rebase 相关操作

use crate::git::commands::GitCommitCommand;
use crate::git::{CommitInfo, GitBranch, GitCommit, GitRepository, GitStash};
use color_eyre::{eyre::WrapErr, Result};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Reword 预览信息
#[derive(Debug, Clone)]
pub struct RewordPreview {
    /// 原始 commit SHA
    pub original_sha: String,
    /// 原始提交消息
    pub original_message: String,
    /// 新提交消息
    pub new_message: String,
    /// 是否是 HEAD
    pub is_head: bool,
    /// 是否已推送到远程
    pub is_pushed: bool,
}

/// 历史 commit reword 选项
#[derive(Debug, Clone)]
pub struct RewordHistoryOptions {
    /// 要修改的 commit SHA
    pub commit_sha: String,
    /// 新的提交消息
    pub new_message: String,
    /// 是否自动 stash
    pub auto_stash: bool,
}

/// 历史 commit reword 结果
#[derive(Debug, Clone)]
pub struct RewordHistoryResult {
    /// 是否成功
    pub success: bool,
    /// 是否有冲突
    pub has_conflicts: bool,
    /// 是否进行了 stash
    pub was_stashed: bool,
}

/// Commit Reword 业务逻辑
pub struct CommitReword;

impl CommitReword {
    /// 格式化 commit 信息为字符串
    ///
    /// # 参数
    ///
    /// * `commit_info` - Commit 信息
    /// * `branch` - 分支名
    ///
    /// # 返回
    ///
    /// 返回格式化的字符串。
    pub fn format_commit_info(commit_info: &CommitInfo, branch: &str) -> String {
        format!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n                         Current Commit Info\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n  Commit SHA:    {}\n  Message:       {}\n  Author:        {}\n  Date:          {}\n  Branch:        {}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            &commit_info.sha[..8],
            commit_info.message,
            commit_info.author,
            commit_info.date,
            branch
        )
    }

    /// 创建 reword 预览信息
    ///
    /// # 参数
    ///
    /// * `commit_info` - Commit 信息
    /// * `new_message` - 新提交消息
    /// * `is_head` - 是否是 HEAD
    /// * `current_branch` - 当前分支名
    ///
    /// # 返回
    ///
    /// 返回 reword 预览信息。
    pub fn create_preview(
        commit_info: &CommitInfo,
        new_message: &str,
        is_head: bool,
        current_branch: &str,
    ) -> Result<RewordPreview> {
        let is_pushed =
            GitBranch::is_commit_in_remote(current_branch, &commit_info.sha).unwrap_or(false);

        Ok(RewordPreview {
            original_sha: commit_info.sha.clone(),
            original_message: commit_info.message.clone(),
            new_message: new_message.to_string(),
            is_head,
            is_pushed,
        })
    }

    /// 格式化 reword 预览信息为字符串
    ///
    /// # 参数
    ///
    /// * `preview` - Reword 预览信息
    ///
    /// # 返回
    ///
    /// 返回格式化的字符串。
    pub fn format_preview(preview: &RewordPreview) -> String {
        let new_sha_text = if preview.is_head {
            "(will be regenerated)"
        } else {
            "(will be modified via rebase)"
        };

        let operation_type = if preview.is_head {
            "Reword HEAD (amend)"
        } else {
            "Reword history commit (rebase)"
        };

        let mut result = format!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n                         Commit Reword Preview\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n  Original Commit SHA:  {}\n  New Commit SHA:       {}\n\n  Original message:     {}\n  New message:          {}\n\n  Operation type:       {}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            &preview.original_sha[..8],
            new_sha_text,
            preview.original_message,
            preview.new_message,
            operation_type
        );

        if preview.is_pushed {
            result.push_str(
                "\n\n⚠️  Warning: This commit may have been pushed to remote\n\nAfter reword, you'll need to force push to update the remote branch:\n  git push --force\n\nThis may affect other collaborators. Please ensure team members are notified.\n",
            );
        }

        result
    }

    /// 检查是否需要显示 force push 警告
    ///
    /// # 参数
    ///
    /// * `current_branch` - 当前分支名
    /// * `old_sha` - 原始 commit SHA
    ///
    /// # 返回
    ///
    /// 如果已推送，返回 `true`。
    pub fn should_show_force_push_warning(current_branch: &str, old_sha: &str) -> Result<bool> {
        GitBranch::is_commit_in_remote(current_branch, old_sha)
    }

    /// 生成完成提示信息
    ///
    /// # 参数
    ///
    /// * `current_branch` - 当前分支名
    /// * `old_sha` - 原始 commit SHA
    ///
    /// # 返回
    ///
    /// 如果已推送，返回提示信息字符串；否则返回 `None`。
    pub fn format_completion_message(
        current_branch: &str,
        old_sha: &str,
    ) -> Result<Option<String>> {
        let is_pushed = Self::should_show_force_push_warning(current_branch, old_sha)?;

        if is_pushed {
            Ok(Some("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n                        Commit Reword Complete\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n  ✓ Commit message has been modified\n\n  Note:\n    - If this commit has been pushed to remote, you need to force push:\n      git push --force\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string()))
        } else {
            Ok(None)
        }
    }

    /// 使用 git 命令执行 reword 操作
    ///
    /// # 参数
    ///
    /// * `parent_sha` - 父 commit SHA（rebase 起点）
    /// * `target_commit_sha` - 要修改消息的 commit SHA
    /// * `new_message` - 新的提交消息
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    fn execute_rebase_reword(
        parent_sha: &str,
        target_commit_sha: &str,
        new_message: &str,
    ) -> Result<()> {
        let repo = GitRepository::open()?;
        let repo_path = repo.path();

        // 验证 parent_sha 和 target_commit_sha 都是有效的 commit
        GitCommitCommand::verify_ref(parent_sha, Some(repo_path))
            .wrap_err_with(|| format!("Invalid parent commit SHA: {}", parent_sha))?;
        GitCommitCommand::verify_ref(target_commit_sha, Some(repo_path))
            .wrap_err_with(|| format!("Invalid target commit SHA: {}", target_commit_sha))?;

        // 获取从 parent_sha 到 HEAD 的所有 commits（按时间顺序，从旧到新）
        let all_commits =
            GitCommitCommand::rev_list(&format!("{}..HEAD", parent_sha), Some(repo_path))
                .wrap_err("Failed to get commit list")?;

        let all_commit_shas: Vec<String> = all_commits
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 构建 rebase todo 列表
        // 目标 commit 使用 "reword"，其他 commits 使用 "pick"
        let mut todo_lines = Vec::new();
        for commit_sha in &all_commit_shas {
            if commit_sha == target_commit_sha {
                todo_lines.push(format!("reword {}", commit_sha));
            } else {
                todo_lines.push(format!("pick {}", commit_sha));
            }
        }

        // 创建临时文件用于 rebase todo 列表
        let mut todo_file =
            NamedTempFile::new().wrap_err("Failed to create temporary file for rebase todo")?;
        let todo_content = todo_lines.join("\n");
        todo_file
            .write_all(todo_content.as_bytes())
            .wrap_err("Failed to write rebase todo")?;
        let todo_path = todo_file.path().to_path_buf();
        todo_file.persist(&todo_path).wrap_err("Failed to persist todo file")?;

        // 创建临时文件用于提交消息
        let mut message_file =
            NamedTempFile::new().wrap_err("Failed to create temporary file for commit message")?;
        message_file
            .write_all(new_message.as_bytes())
            .wrap_err("Failed to write commit message")?;
        let message_path = message_file.path().to_path_buf();
        message_file.persist(&message_path).wrap_err("Failed to persist message file")?;

        // 创建序列编辑器脚本（用于自动编辑 rebase todo）
        let (seq_editor_script, seq_editor_ext) = if cfg!(windows) {
            (
                format!(
                    r#"@echo off
copy /Y "{}" "%1" >nul
"#,
                    todo_path.to_string_lossy().replace('/', "\\")
                ),
                ".bat",
            )
        } else {
            (
                format!(
                    r#"#!/bin/sh
cp "{}" "$1"
"#,
                    todo_path.to_string_lossy()
                ),
                ".sh",
            )
        };

        let mut seq_editor_file = NamedTempFile::with_suffix(seq_editor_ext)
            .wrap_err("Failed to create temporary file for sequence editor")?;
        seq_editor_file
            .write_all(seq_editor_script.as_bytes())
            .wrap_err("Failed to write sequence editor script")?;
        let seq_editor_path = seq_editor_file.path().to_path_buf();
        seq_editor_file
            .persist(&seq_editor_path)
            .wrap_err("Failed to persist sequence editor file")?;

        // 设置脚本可执行权限（Unix）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&seq_editor_path, std::fs::Permissions::from_mode(0o755))
                .wrap_err("Failed to set executable permissions")?;
        }

        // 创建提交消息编辑器脚本
        let (commit_editor_script, commit_editor_ext) = if cfg!(windows) {
            (
                format!(
                    r#"@echo off
copy /Y "{}" "%1" >nul
"#,
                    message_path.to_string_lossy().replace('/', "\\")
                ),
                ".bat",
            )
        } else {
            (
                format!(
                    r#"#!/bin/sh
cp "{}" "$1"
"#,
                    message_path.to_string_lossy()
                ),
                ".sh",
            )
        };

        let mut commit_editor_file = NamedTempFile::with_suffix(commit_editor_ext)
            .wrap_err("Failed to create temporary file for commit editor")?;
        commit_editor_file
            .write_all(commit_editor_script.as_bytes())
            .wrap_err("Failed to write commit editor script")?;
        let commit_editor_path = commit_editor_file.path().to_path_buf();
        commit_editor_file
            .persist(&commit_editor_path)
            .wrap_err("Failed to persist commit editor file")?;

        // 设置脚本可执行权限（Unix）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&commit_editor_path, std::fs::Permissions::from_mode(0o755))
                .wrap_err("Failed to set executable permissions")?;
        }

        // 执行 rebase
        // 使用 GIT_SEQUENCE_EDITOR 和 GIT_EDITOR 环境变量来自动化交互
        let mut rebase_cmd = Command::new("git");
        rebase_cmd
            .arg("rebase")
            .arg("-i")
            .arg(parent_sha)
            .current_dir(repo_path)
            .env("GIT_SEQUENCE_EDITOR", &seq_editor_path)
            .env("GIT_EDITOR", &commit_editor_path)
            .env("GIT_TERMINAL_PROMPT", "0");

        let rebase_output = rebase_cmd.output().wrap_err("Failed to execute git rebase")?;

        // 清理临时文件
        let _ = std::fs::remove_file(&todo_path);
        let _ = std::fs::remove_file(&message_path);
        let _ = std::fs::remove_file(&seq_editor_path);
        let _ = std::fs::remove_file(&commit_editor_path);

        if !rebase_output.status.success() {
            let stderr = String::from_utf8_lossy(&rebase_output.stderr);
            let stdout = String::from_utf8_lossy(&rebase_output.stdout);

            // 检查是否有冲突
            if stderr.contains("conflict") || stdout.contains("conflict") {
                color_eyre::eyre::bail!(
                    "Rebase conflicts detected. Please resolve manually:\n  1. Review conflicted files\n  2. Resolve conflicts\n  3. Stage resolved files: git add <files>\n  4. Continue rebase: git rebase --continue\n  5. Or abort rebase: git rebase --abort"
                );
            }

            return Err(color_eyre::eyre::eyre!(
                "Failed to execute rebase: {}\n{}",
                stderr,
                stdout
            ));
        }

        Ok(())
    }

    /// 执行历史 commit reword（核心业务逻辑）
    ///
    /// # 参数
    ///
    /// * `options` - Reword 选项
    ///
    /// # 返回
    ///
    /// 返回 reword 结果。
    pub fn reword_history_commit(options: RewordHistoryOptions) -> Result<RewordHistoryResult> {
        // 步骤1: 检查工作区状态，如果有未提交的更改，需要 stash
        let has_stashed = if options.auto_stash && GitCommit::has_commit()? {
            GitStash::stash_push(Some("Auto-stash before reword history commit"))?;
            true
        } else {
            false
        };

        // 步骤2: 找到目标 commit 的父 commit（rebase 起点）
        let parent_sha = match GitCommit::get_parent_commit(&options.commit_sha) {
            Ok(sha) => sha,
            Err(e) => {
                // 如果是根 commit，无法 rebase
                if has_stashed {
                    let _ = GitStash::stash_pop(None);
                }
                color_eyre::eyre::bail!(
                    "Cannot reword root commit (commit has no parent). Error: {}",
                    e
                );
            }
        };

        // 步骤3: 验证目标 commit 存在
        let repo = GitRepository::open()?;
        GitCommitCommand::verify_ref(&options.commit_sha, Some(repo.path()))
            .wrap_err_with(|| format!("Commit not found: {}", options.commit_sha))?;

        // 步骤4: 使用 git2 rebase API 执行 reword
        let rebase_result =
            Self::execute_rebase_reword(&parent_sha, &options.commit_sha, &options.new_message);

        // 步骤9: 处理 rebase 结果
        match rebase_result {
            Ok(()) => {
                // 恢复 stash（如果有）
                if has_stashed {
                    let _ = GitStash::stash_pop(None);
                }
                Ok(RewordHistoryResult {
                    success: true,
                    has_conflicts: false,
                    was_stashed: has_stashed,
                })
            }
            Err(e) => {
                // 如果 rebase 失败，恢复 stash（如果有）
                if has_stashed {
                    let _ = GitStash::stash_pop(None);
                }

                // 检查是否是 rebase 冲突
                let error_msg = e.to_string().to_lowercase();
                let has_conflicts =
                    error_msg.contains("conflict") || error_msg.contains("could not apply");

                if has_conflicts {
                    Err(e).wrap_err_with(|| {
                        "Rebase conflicts detected. Please resolve manually:\n  1. Review conflicted files\n  2. Resolve conflicts\n  3. Stage resolved files: git add <files>\n  4. Continue rebase: git rebase --continue\n  5. Or abort rebase: git rebase --abort"
                    })
                } else {
                    Err(e).wrap_err_with(|| "Failed to execute rebase")
                }
            }
        }
    }
}
