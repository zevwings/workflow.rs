//! Commit Squash 业务逻辑
//!
//! 提供 squash 操作相关的业务逻辑，包括：
//! - 获取当前分支创建之后的提交
//! - 预览信息生成
//! - 格式化显示
//! - Rebase 相关操作

use crate::git::commands::GitCommitCommand;
use crate::git::{CommitInfo, GitBranch, GitCommit, GitRepository, GitStash};
use color_eyre::{eyre::WrapErr, Result};
use std::collections::HashSet;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

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
        let base_branch = GitBranch::detect_base_branch(current_branch, &default_branch)
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

    /// 使用 git 命令执行 squash 操作
    ///
    /// # 参数
    ///
    /// * `base_sha` - 基础 commit SHA（rebase 起点）
    /// * `selected_commit_shas` - 要压缩的 commit SHA 列表（按时间顺序，从旧到新）
    /// * `new_message` - 新的提交消息
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    fn execute_rebase_squash(
        base_sha: &str,
        selected_commit_shas: &[String],
        new_message: &str,
    ) -> Result<()> {
        let repo = GitRepository::open()?;
        let repo_path = repo.path();

        // 验证 base_sha 和 selected_commit_shas 都是有效的 commit
        GitCommitCommand::verify_ref(base_sha, Some(repo_path))
            .wrap_err_with(|| format!("Invalid base commit SHA: {}", base_sha))?;

        for commit_sha in selected_commit_shas {
            GitCommitCommand::verify_ref(commit_sha, Some(repo_path))
                .wrap_err_with(|| format!("Invalid commit SHA: {}", commit_sha))?;
        }

        // 获取从 base_sha 到 HEAD 的所有 commits（按时间顺序，从旧到新）
        let all_commits =
            GitCommitCommand::rev_list(&format!("{}..HEAD", base_sha), Some(repo_path))
                .wrap_err("Failed to get commit list")?;

        let all_commit_shas: Vec<String> = all_commits
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 将选中的 SHA 转换为集合，方便查找
        let selected_set: HashSet<String> = selected_commit_shas.iter().cloned().collect();

        // 构建 rebase todo 列表
        // 第一个选中的 commit 使用 "pick"，后续的选中 commits 使用 "squash"
        let mut todo_lines = Vec::new();
        let mut is_first_selected = true;

        for commit_sha in &all_commit_shas {
            if selected_set.contains(commit_sha) {
                if is_first_selected {
                    todo_lines.push(format!("pick {}", commit_sha));
                    is_first_selected = false;
                } else {
                    todo_lines.push(format!("squash {}", commit_sha));
                }
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
        // 使用跨平台的方式：在 Unix 上使用 sh，在 Windows 上使用 cmd
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
            .arg(base_sha)
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

        // 步骤3: 验证选中的 commits 存在
        let repo = GitRepository::open()?;
        for commit_sha in &options.commit_shas {
            GitCommitCommand::verify_ref(commit_sha, Some(repo.path()))
                .wrap_err_with(|| format!("Commit not found: {}", commit_sha))?;
        }

        // 步骤4: 使用 git2 rebase API 执行 squash
        let rebase_result =
            Self::execute_rebase_squash(&base_sha, &options.commit_shas, &options.new_message);

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
