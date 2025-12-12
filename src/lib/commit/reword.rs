//! Commit Reword 业务逻辑
//!
//! 提供 reword 操作相关的业务逻辑，包括：
//! - 预览信息生成
//! - 格式化显示
//! - 历史 commit reword 执行
//! - Rebase 相关操作

use crate::git::{CommitInfo, GitBranch, GitCommit, GitStash};
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Rebase 编辑器配置
#[derive(Debug, Clone)]
struct RebaseEditorConfig {
    /// Sequence editor 脚本路径
    sequence_editor_script: PathBuf,
    /// Message editor 脚本路径
    message_editor_script: PathBuf,
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

    /// 创建 rebase todo 文件内容
    ///
    /// 将目标 commit 标记为指定的操作（如 `reword`），其他 commits 保持 `pick`。
    ///
    /// # 参数
    ///
    /// * `commits` - Commits 列表
    /// * `target_commit_sha` - 目标 commit SHA
    /// * `action` - 对目标 commit 执行的操作（如 "reword", "pick" 等）
    ///
    /// # 返回
    ///
    /// 返回 rebase todo 文件内容。
    fn create_rebase_todo(
        commits: &[CommitInfo],
        target_commit_sha: &str,
        action: &str,
    ) -> Result<String> {
        let mut todo_lines = Vec::new();

        for commit in commits {
            let commit_action = if commit.sha == target_commit_sha {
                action
            } else {
                "pick"
            };
            // rebase todo 格式: action sha message
            // 注意：消息可能包含特殊字符，需要适当处理
            let message = commit.message.replace('\n', " ");
            todo_lines.push(format!(
                "{} {} {}",
                commit_action,
                &commit.sha[..12],
                message
            ));
        }

        Ok(todo_lines.join("\n"))
    }

    /// 创建 rebase 编辑器脚本
    ///
    /// # 参数
    ///
    /// * `todo_file` - Rebase todo 文件路径
    /// * `message_file` - Commit 消息文件路径
    ///
    /// # 返回
    ///
    /// 返回编辑器配置。
    fn create_rebase_editor_scripts(
        todo_file: &Path,
        message_file: &Path,
    ) -> Result<RebaseEditorConfig> {
        let temp_dir = std::env::temp_dir();

        // 创建脚本来自动编辑 rebase todo 文件
        // Git 会将 todo 文件路径作为第一个参数 ($1) 传递给编辑器
        let script_content = format!(
            r#"#!/bin/sh
# 自动编辑 rebase todo 文件
cp "{}" "$1"
"#,
            todo_file.to_string_lossy().replace('\\', "/")
        );

        let sequence_editor_script =
            temp_dir.join(format!("workflow-sequence-editor-{}", std::process::id()));
        fs::write(&sequence_editor_script, script_content)
            .with_context(|| "Failed to write sequence editor script")?;

        // 设置脚本可执行权限（仅 Unix 系统）
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&sequence_editor_script)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&sequence_editor_script, perms)?;
        }

        // 创建脚本来自动提供新消息
        // Git 会将 commit message 文件路径作为第一个参数 ($1) 传递给编辑器
        let message_script_content = format!(
            r#"#!/bin/sh
# 自动提供新 commit 消息
cp "{}" "$1"
"#,
            message_file.to_string_lossy().replace('\\', "/")
        );

        let message_editor_script =
            temp_dir.join(format!("workflow-message-editor-{}", std::process::id()));
        fs::write(&message_editor_script, message_script_content)
            .with_context(|| "Failed to write message editor script")?;

        // 设置脚本可执行权限（仅 Unix 系统）
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&message_editor_script)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&message_editor_script, perms)?;
        }

        Ok(RebaseEditorConfig {
            sequence_editor_script,
            message_editor_script,
        })
    }

    /// 执行 rebase，使用自定义编辑器脚本
    ///
    /// # 参数
    ///
    /// * `parent_sha` - 父 commit SHA（rebase 起点）
    /// * `config` - Rebase 编辑器配置
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    fn execute_rebase_with_editors(parent_sha: &str, config: &RebaseEditorConfig) -> Result<()> {
        // 执行 rebase
        // GIT_SEQUENCE_EDITOR: 用于编辑 rebase todo 文件
        // GIT_EDITOR: 用于编辑 commit 消息（当遇到 reword 时）
        let rebase_result = Command::new("git")
            .arg("rebase")
            .arg("-i")
            .arg(parent_sha)
            .env("GIT_SEQUENCE_EDITOR", &config.sequence_editor_script)
            .env("GIT_EDITOR", &config.message_editor_script)
            .output()
            .with_context(|| "Failed to execute git rebase")?;

        if !rebase_result.status.success() {
            let stderr = String::from_utf8_lossy(&rebase_result.stderr);
            let stdout = String::from_utf8_lossy(&rebase_result.stdout);
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            anyhow::bail!("Rebase failed: {}", error_msg);
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
                anyhow::bail!(
                    "Cannot reword root commit (commit has no parent). Error: {}",
                    e
                );
            }
        };

        // 步骤3: 获取从父 commit 到 HEAD 的所有 commits
        let commits = GitCommit::get_commits_from_to_head(&parent_sha)
            .with_context(|| "Failed to get commits for rebase")?;

        if commits.is_empty() {
            if has_stashed {
                let _ = GitStash::stash_pop(None);
            }
            anyhow::bail!("No commits found between parent and HEAD");
        }

        // 步骤4: 创建 rebase todo 文件
        let todo_content = Self::create_rebase_todo(&commits, &options.commit_sha, "reword")?;

        // 步骤5: 创建临时文件用于 rebase todo
        let temp_dir = std::env::temp_dir();
        let todo_file = temp_dir.join(format!("workflow-rebase-todo-{}", std::process::id()));
        fs::write(&todo_file, &todo_content).with_context(|| "Failed to write rebase todo file")?;

        // 步骤6: 创建临时文件用于新消息
        let message_file = temp_dir.join(format!("workflow-commit-message-{}", std::process::id()));
        fs::write(&message_file, &options.new_message)
            .with_context(|| "Failed to write commit message file")?;

        // 步骤7: 创建编辑器脚本
        let editor_config = Self::create_rebase_editor_scripts(&todo_file, &message_file)?;

        // 步骤8: 执行 rebase
        let rebase_result = Self::execute_rebase_with_editors(&parent_sha, &editor_config);

        // 清理临时文件
        let _ = fs::remove_file(&todo_file);
        let _ = fs::remove_file(&message_file);
        let _ = fs::remove_file(&editor_config.sequence_editor_script);
        let _ = fs::remove_file(&editor_config.message_editor_script);

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
                    Err(e).with_context(|| {
                        "Rebase conflicts detected. Please resolve manually:\n  1. Review conflicted files\n  2. Resolve conflicts\n  3. Stage resolved files: git add <files>\n  4. Continue rebase: git rebase --continue\n  5. Or abort rebase: git rebase --abort"
                    })
                } else {
                    Err(e).with_context(|| "Failed to execute rebase")
                }
            }
        }
    }
}
