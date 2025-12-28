//! Commit Reword 业务逻辑
//!
//! 提供 reword 操作相关的业务逻辑，包括：
//! - 预览信息生成
//! - 格式化显示
//! - 历史 commit reword 执行
//! - Rebase 相关操作

use crate::git::{CommitInfo, GitBranch, GitCommit, GitRepository, GitStash};
use color_eyre::{eyre::WrapErr, Result};
use git2::Oid;

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

    /// 使用 git2 rebase API 执行 reword 操作
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

        // 解析 commit SHA
        let parent_oid = Oid::from_str(parent_sha)
            .wrap_err_with(|| format!("Invalid parent commit SHA: {}", parent_sha))?;
        let target_oid = Oid::from_str(target_commit_sha)
            .wrap_err_with(|| format!("Invalid target commit SHA: {}", target_commit_sha))?;

        // 获取 HEAD commit（在获取可变引用之前）
        let head_commit_id = {
            let head = repo.head()?;
            let head_commit = head.peel_to_commit().wrap_err("Failed to get commit from HEAD")?;
            head_commit.id()
        };

        // 转换为 AnnotatedCommit（rebase 需要）
        let mut repo = repo;
        let repo_inner = repo.as_inner_mut();
        let head_annotated = repo_inner
            .find_annotated_commit(head_commit_id)
            .wrap_err("Failed to create annotated commit from HEAD")?;
        let parent_annotated =
            repo_inner.find_annotated_commit(parent_oid).wrap_err_with(|| {
                format!(
                    "Failed to create annotated commit from parent: {}",
                    parent_sha
                )
            })?;

        // 初始化 rebase
        let mut rebase_opts = git2::RebaseOptions::new();
        // 不使用 inmemory 模式，因为我们需要修改工作目录

        let mut rebase = repo_inner
            .rebase(
                Some(&head_annotated),
                Some(&parent_annotated),
                None,
                Some(&mut rebase_opts),
            )
            .wrap_err_with(|| format!("Failed to initialize rebase from parent: {}", parent_sha))?;

        // 获取签名
        let signature = repo_inner.signature().wrap_err("Failed to get signature")?;

        // 应用所有 rebase 操作
        while let Some(op) = rebase.next() {
            let op = op.wrap_err("Failed to get next rebase operation")?;

            // 检查是否有冲突
            if repo_inner.index()?.has_conflicts() {
                rebase.abort().ok();
                color_eyre::eyre::bail!(
                    "Rebase conflicts detected. Please resolve manually:\n  1. Review conflicted files\n  2. Resolve conflicts\n  3. Stage resolved files: git add <files>\n  4. Continue rebase: git rebase --continue\n  5. Or abort rebase: git rebase --abort"
                );
            }

            // 如果是目标 commit，需要手动创建新的 commit 使用新消息
            if op.id() == target_oid {
                // 获取当前 index 的 tree
                let mut index = repo_inner.index()?;
                let tree_id = index.write_tree().wrap_err("Failed to write tree")?;
                let tree = repo_inner.find_tree(tree_id).wrap_err("Failed to find tree")?;

                // 获取 rebase 的当前 HEAD（父 commit）
                let head_ref = repo_inner.find_reference("HEAD").ok();
                let parent_commit = head_ref.and_then(|r| r.peel_to_commit().ok());

                // 创建新的 commit，使用新消息
                let new_commit_oid = if let Some(parent) = parent_commit.as_ref() {
                    repo_inner.commit(None, &signature, &signature, new_message, &tree, &[parent])
                } else {
                    // 如果没有父 commit（根 commit），创建没有父的 commit
                    repo_inner.commit(None, &signature, &signature, new_message, &tree, &[])
                }
                .wrap_err_with(|| {
                    format!(
                        "Failed to create commit with new message for: {}",
                        target_commit_sha
                    )
                })?;

                // 更新 HEAD 指向新创建的 commit
                repo_inner.set_head_detached(new_commit_oid).wrap_err("Failed to update HEAD")?;

                // 注意：我们已经手动创建了 commit，但 rebase 仍然需要知道操作已完成
                // 由于 git2 rebase API 的限制，我们需要使用 rebase.commit() 来标记操作完成
                // 但这里传入 None 会使用当前的 HEAD（我们刚创建的 commit）
                // 实际上，由于我们已经更新了 HEAD，rebase.commit() 应该会使用新的 commit
                // 但为了安全，我们仍然调用 rebase.commit() 来继续 rebase 流程
                rebase
                    .commit(None, &signature, None)
                    .wrap_err("Failed to commit rebase operation after creating new commit")?;
            } else {
                // 对于非目标 commit，正常提交
                rebase
                    .commit(None, &signature, None)
                    .wrap_err("Failed to commit rebase operation")?;
            }
        }

        // 完成 rebase
        rebase.finish(None).wrap_err_with(|| {
            format!("Failed to finish rebase for reword: {}", target_commit_sha)
        })?;

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
        let target_oid = git2::Oid::from_str(&options.commit_sha)
            .wrap_err_with(|| format!("Invalid commit SHA: {}", options.commit_sha))?;

        // 检查 commit 是否存在
        repo.as_inner()
            .find_commit(target_oid)
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
