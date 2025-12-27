//! Commit Squash 业务逻辑
//!
//! 提供 squash 操作相关的业务逻辑，包括：
//! - 获取当前分支创建之后的提交
//! - 预览信息生成
//! - 格式化显示
//! - Rebase 相关操作

use crate::base::constants::errors::file_operations;
use crate::base::fs::FileWriter;
use crate::git::{CommitInfo, GitBranch, GitCommit, GitStash};
use color_eyre::{eyre::WrapErr, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Git 环境变量常量
const GIT_SEQUENCE_EDITOR: &str = "GIT_SEQUENCE_EDITOR";
const GIT_EDITOR: &str = "GIT_EDITOR";

/// Squash 预览信息
#[derive(Debug, Clone)]
pub struct SquashPreview {
    /// 要压缩的 commits 列表
    pub commits: Vec<CommitInfo>,
    /// 新的提交消息
    pub new_message: String,
    /// 基础 commit SHA（压缩起点）
    pub base_sha: String,
    /// 是否已推送到远程
    pub is_pushed: bool,
}

/// Squash 选项
#[derive(Debug, Clone)]
pub struct SquashOptions {
    /// 要压缩的 commit SHA 列表（按时间顺序，从旧到新）
    pub commit_shas: Vec<String>,
    /// 新的提交消息
    pub new_message: String,
    /// 是否自动 stash
    pub auto_stash: bool,
}

/// Squash 结果
#[derive(Debug, Clone)]
pub struct SquashResult {
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

/// Commit Squash 业务逻辑
pub struct CommitSquash;

impl CommitSquash {
    /// 获取当前分支创建之后的提交
    ///
    /// 通过检测当前分支基于哪个分支创建，然后获取该分支之后的所有提交。
    ///
    /// # 参数
    ///
    /// * `current_branch` - 当前分支名称
    ///
    /// # 返回
    ///
    /// 返回当前分支创建之后的提交列表（从旧到新）。
    pub fn get_branch_commits(current_branch: &str) -> Result<Vec<CommitInfo>> {
        // 1. 获取默认分支
        let default_branch =
            GitBranch::get_default_branch().wrap_err("Failed to get default branch")?;

        // 2. 尝试检测当前分支基于哪个分支创建
        let base_branch =
            crate::commands::pr::helpers::detect_base_branch(current_branch, &default_branch)
                .wrap_err("Failed to detect base branch")?;

        // 3. 确定基础分支（优先使用检测到的分支，否则使用默认分支）
        let actual_base = base_branch.as_deref().unwrap_or(&default_branch);

        // 4. 获取从基础分支到当前分支的所有提交
        let commit_shas = GitBranch::get_commits_between(actual_base, current_branch)
            .wrap_err_with(|| {
                format!(
                    "Failed to get commits between '{}' and '{}'",
                    actual_base, current_branch
                )
            })?;

        if commit_shas.is_empty() {
            return Ok(Vec::new());
        }

        // 5. 获取每个 commit 的详细信息
        let mut commits = Vec::new();
        for sha in commit_shas {
            let commit_info = GitCommit::get_commit_info(&sha)
                .wrap_err_with(|| format!("Failed to get commit info: {}", &sha[..8]))?;
            commits.push(commit_info);
        }

        Ok(commits)
    }

    /// 创建 squash 预览信息
    ///
    /// # 参数
    ///
    /// * `commits` - 要压缩的 commits 列表
    /// * `new_message` - 新的提交消息
    /// * `current_branch` - 当前分支名
    ///
    /// # 返回
    ///
    /// 返回 squash 预览信息。
    pub fn create_preview(
        commits: &[CommitInfo],
        new_message: &str,
        current_branch: &str,
    ) -> Result<SquashPreview> {
        if commits.is_empty() {
            color_eyre::eyre::bail!("No commits to squash");
        }

        // 获取基础 commit SHA（第一个要压缩的 commit 的父 commit）
        let base_sha = if commits.len() == 1 {
            // 如果只有一个 commit，获取它的父 commit
            GitCommit::get_parent_commit(&commits[0].sha).wrap_err("Failed to get parent commit")?
        } else {
            // 如果有多个 commits，获取第一个 commit 的父 commit
            GitCommit::get_parent_commit(&commits[0].sha).wrap_err("Failed to get parent commit")?
        };

        // 检查是否已推送（检查第一个 commit 是否在远程）
        let is_pushed =
            GitBranch::is_commit_in_remote(current_branch, &commits[0].sha).unwrap_or(false);

        Ok(SquashPreview {
            commits: commits.to_vec(),
            new_message: new_message.to_string(),
            base_sha,
            is_pushed,
        })
    }

    /// 格式化 squash 预览信息为字符串
    ///
    /// # 参数
    ///
    /// * `preview` - Squash 预览信息
    ///
    /// # 返回
    ///
    /// 返回格式化的字符串。
    pub fn format_preview(preview: &SquashPreview) -> String {
        let mut result = format!(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n                         Commit Squash Preview\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n  Commits to squash:  {} commit(s)\n  New commit message: {}\n\n  Commits:\n",
            preview.commits.len(),
            preview.new_message
        );

        for (idx, commit) in preview.commits.iter().enumerate() {
            result.push_str(&format!(
                "    {}. [{}] {}\n",
                idx + 1,
                &commit.sha[..8],
                commit.message
            ));
        }

        result.push_str(
            "\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
        );

        if preview.is_pushed {
            result.push_str(
                "\n\n⚠️  Warning: Some commits may have been pushed to remote\n\nAfter squash, you'll need to force push to update the remote branch:\n  git push --force\n\nThis may affect other collaborators. Please ensure team members are notified.\n",
            );
        }

        result
    }

    /// 创建 rebase todo 文件内容
    ///
    /// 将选中的 commits 标记为 `squash`，第一个选中的 commit 标记为 `pick`。
    ///
    /// # 参数
    ///
    /// * `commits` - 所有 commits 列表（从旧到新）
    /// * `selected_commit_shas` - 要压缩的 commit SHA 列表（按时间顺序，从旧到新）
    ///
    /// # 返回
    ///
    /// 返回 rebase todo 文件内容。
    fn create_rebase_todo(
        commits: &[CommitInfo],
        selected_commit_shas: &[String],
    ) -> Result<String> {
        if selected_commit_shas.is_empty() {
            color_eyre::eyre::bail!("No commits selected for squash");
        }

        // 将选中的 SHA 转换为 HashSet 以便快速查找
        let selected_set: std::collections::HashSet<&str> =
            selected_commit_shas.iter().map(|s| s.as_str()).collect();

        let mut todo_lines = Vec::new();
        let mut first_selected = true;

        for commit in commits {
            if selected_set.contains(commit.sha.as_str()) {
                // 第一个选中的 commit 使用 pick，其他的使用 squash
                let action = if first_selected {
                    first_selected = false;
                    "pick"
                } else {
                    "squash"
                };
                let message = commit.message.replace('\n', " ");
                todo_lines.push(format!("{} {} {}", action, &commit.sha[..12], message));
            } else {
                // 未选中的 commit 保持 pick
                let message = commit.message.replace('\n', " ");
                todo_lines.push(format!("pick {} {}", &commit.sha[..12], message));
            }
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
        let script_content = format!(
            r#"#!/bin/sh
# 自动编辑 rebase todo 文件
cp "{}" "$1"
"#,
            todo_file.to_string_lossy().replace('\\', "/")
        );

        let sequence_editor_script = temp_dir.join(format!(
            "workflow-squash-sequence-editor-{}",
            std::process::id()
        ));
        FileWriter::new(&sequence_editor_script)
            .write_str(&script_content)
            .wrap_err_with(|| file_operations::WRITE_SEQUENCE_EDITOR_SCRIPT_FAILED)?;

        // 设置脚本可执行权限（仅 Unix 系统）
        #[cfg(unix)]
        {
            FileWriter::new(&sequence_editor_script)
                .set_permissions(0o755)
                .wrap_err_with(|| file_operations::WRITE_SEQUENCE_EDITOR_SCRIPT_FAILED)?;
        }

        // 创建脚本来自动提供新消息
        let message_script_content = format!(
            r#"#!/bin/sh
# 自动提供新 commit 消息
cp "{}" "$1"
"#,
            message_file.to_string_lossy().replace('\\', "/")
        );

        let message_editor_script = temp_dir.join(format!(
            "workflow-squash-message-editor-{}",
            std::process::id()
        ));
        FileWriter::new(&message_editor_script)
            .write_str(&message_script_content)
            .wrap_err_with(|| file_operations::WRITE_MESSAGE_EDITOR_SCRIPT_FAILED)?;

        // 设置脚本可执行权限（仅 Unix 系统）
        #[cfg(unix)]
        {
            FileWriter::new(&message_editor_script)
                .set_permissions(0o755)
                .wrap_err_with(|| file_operations::WRITE_MESSAGE_EDITOR_SCRIPT_FAILED)?;
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
    /// * `base_sha` - 基础 commit SHA（rebase 起点）
    /// * `config` - Rebase 编辑器配置
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    fn execute_rebase_with_editors(base_sha: &str, config: &RebaseEditorConfig) -> Result<()> {
        // 执行 rebase
        let rebase_result = Command::new("git")
            .arg("rebase")
            .arg("-i")
            .arg(base_sha)
            .env(GIT_SEQUENCE_EDITOR, &config.sequence_editor_script)
            .env(GIT_EDITOR, &config.message_editor_script)
            .output()
            .wrap_err_with(|| "Failed to execute git rebase")?;

        if !rebase_result.status.success() {
            let stderr = String::from_utf8_lossy(&rebase_result.stderr);
            let stdout = String::from_utf8_lossy(&rebase_result.stdout);
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            color_eyre::eyre::bail!("Rebase failed: {}", error_msg);
        }

        Ok(())
    }

    /// 执行 squash 操作（核心业务逻辑）
    ///
    /// # 参数
    ///
    /// * `options` - Squash 选项
    ///
    /// # 返回
    ///
    /// 返回 squash 结果。
    pub fn execute_squash(options: SquashOptions) -> Result<SquashResult> {
        if options.commit_shas.is_empty() {
            color_eyre::eyre::bail!("No commits selected for squash");
        }

        // 步骤1: 检查工作区状态，如果有未提交的更改，需要 stash
        let has_stashed = if options.auto_stash && GitCommit::has_commit()? {
            GitStash::stash_push(Some("Auto-stash before squash commits"))?;
            true
        } else {
            false
        };

        // 步骤2: 获取第一个要压缩的 commit 的父 commit（rebase 起点）
        let base_sha = match GitCommit::get_parent_commit(&options.commit_shas[0]) {
            Ok(sha) => sha,
            Err(e) => {
                if has_stashed {
                    let _ = GitStash::stash_pop(None);
                }
                color_eyre::eyre::bail!(
                    "Cannot squash root commit (commit has no parent). Error: {}",
                    e
                );
            }
        };

        // 步骤3: 获取从父 commit 到 HEAD 的所有 commits
        let commits = GitCommit::get_commits_from_to_head(&base_sha)
            .wrap_err_with(|| "Failed to get commits for rebase")?;

        if commits.is_empty() {
            if has_stashed {
                let _ = GitStash::stash_pop(None);
            }
            color_eyre::eyre::bail!("No commits found between parent and HEAD");
        }

        // 步骤4: 创建 rebase todo 文件
        let todo_content = Self::create_rebase_todo(&commits, &options.commit_shas)?;

        // 步骤5: 创建临时文件用于 rebase todo
        let temp_dir = std::env::temp_dir();
        let todo_file = temp_dir.join(format!("workflow-squash-todo-{}", std::process::id()));
        FileWriter::new(&todo_file)
            .write_str(&todo_content)
            .wrap_err_with(|| "Failed to write rebase todo file")?;

        // 步骤6: 创建临时文件用于新消息
        let message_file = temp_dir.join(format!("workflow-squash-message-{}", std::process::id()));
        FileWriter::new(&message_file)
            .write_str(&options.new_message)
            .wrap_err_with(|| "Failed to write commit message file")?;

        // 步骤7: 创建编辑器脚本
        let editor_config = Self::create_rebase_editor_scripts(&todo_file, &message_file)?;

        // 步骤8: 执行 rebase
        let rebase_result = Self::execute_rebase_with_editors(&base_sha, &editor_config);

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
                Ok(SquashResult {
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

    /// 检查是否需要显示 force push 警告
    ///
    /// # 参数
    ///
    /// * `current_branch` - 当前分支名
    /// * `commit_shas` - 要压缩的 commit SHA 列表
    ///
    /// # 返回
    ///
    /// 如果任何一个 commit 已推送，返回 `true`。
    pub fn should_show_force_push_warning(
        current_branch: &str,
        commit_shas: &[String],
    ) -> Result<bool> {
        for commit_sha in commit_shas {
            if GitBranch::is_commit_in_remote(current_branch, commit_sha)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 生成完成提示信息
    ///
    /// # 参数
    ///
    /// * `current_branch` - 当前分支名
    /// * `commit_shas` - 要压缩的 commit SHA 列表
    ///
    /// # 返回
    ///
    /// 如果已推送，返回提示信息字符串；否则返回 `None`。
    pub fn format_completion_message(
        current_branch: &str,
        commit_shas: &[String],
    ) -> Result<Option<String>> {
        let is_pushed = Self::should_show_force_push_warning(current_branch, commit_shas)?;

        if is_pushed {
            Ok(Some(
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n                        Commit Squash Complete\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n  ✓ Commits have been squashed\n\n  Note:\n    - If these commits have been pushed to remote, you need to force push:\n      git push --force\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                    .to_string(),
            ))
        } else {
            Ok(None)
        }
    }
}
