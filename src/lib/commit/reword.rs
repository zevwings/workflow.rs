//! Commit Reword 业务逻辑
//!
//! 提供 reword 操作相关的业务逻辑，包括：
//! - 预览信息生成
//! - 格式化显示
//! - 历史 commit reword 执行
//! - Rebase 相关操作

use crate::git::{CommitInfo, GitBranch, GitCommit, GitStash};
use color_eyre::{eyre::WrapErr, Result};
use git2::Repository;

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

    /// 使用 git2 执行 reword rebase
    ///
    /// # 参数
    ///
    /// * `parent_sha` - 父 commit SHA（rebase 起点）
    /// * `commits` - 所有 commits 列表（从旧到新）
    /// * `target_commit_sha` - 要修改消息的 commit SHA
    /// * `new_message` - 新的提交消息
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    fn execute_reword_rebase(
        parent_sha: &str,
        _commits: &[CommitInfo],
        target_commit_sha: &str,
        new_message: &str,
    ) -> Result<()> {
        let repo = Repository::open(".").wrap_err("Failed to open repository")?;

        // 1. 获取父 commit
        let parent_oid = repo
            .revparse_single(parent_sha)
            .wrap_err_with(|| format!("Failed to parse parent commit: {}", parent_sha))?
            .id();
        let parent_annotated = repo
            .find_annotated_commit(parent_oid)
            .wrap_err("Failed to create annotated commit for parent")?;

        // 2. 获取当前 HEAD
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;
        let head_annotated = repo
            .find_annotated_commit(head_commit.id())
            .wrap_err("Failed to create annotated commit for HEAD")?;

        // 3. 初始化 rebase
        let mut rebase_opts = git2::RebaseOptions::new();
        rebase_opts.inmemory(false); // 需要写入到仓库
        let mut rebase = repo
            .rebase(
                Some(&head_annotated),
                Some(&parent_annotated),
                None,
                Some(&mut rebase_opts),
            )
            .wrap_err("Failed to initialize rebase")?;

        // 4. 处理每个 rebase 操作
        while let Some(op) = rebase.next() {
            let op = op?;
            let commit_oid = op.id();
            let commit_sha = commit_oid.to_string();
            let commit = repo.find_commit(commit_oid)?;
            let author = commit.author();

            if commit_sha == target_commit_sha {
                // 这是要 reword 的 commit，使用新消息
                rebase
                    .commit(Some(&author), &author, Some(new_message))
                    .wrap_err("Failed to commit reword operation")?;
            } else {
                // 其他 commit，保持原消息
                rebase
                    .commit(Some(&author), &author, None)
                    .wrap_err("Failed to commit rebase operation")?;
            }
        }

        // 5. 完成 rebase
        rebase.finish(None).wrap_err("Failed to finish rebase")?;

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

        // 步骤3: 获取从父 commit 到 HEAD 的所有 commits
        let commits = GitCommit::get_commits_from_to_head(&parent_sha)
            .wrap_err_with(|| "Failed to get commits for rebase")?;

        if commits.is_empty() {
            if has_stashed {
                let _ = GitStash::stash_pop(None);
            }
            color_eyre::eyre::bail!("No commits found between parent and HEAD");
        }

        // 步骤4: 使用 git2 执行 reword rebase
        let rebase_result = Self::execute_reword_rebase(
            &parent_sha,
            &commits,
            &options.commit_sha,
            &options.new_message,
        );

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
