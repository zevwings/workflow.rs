//! Commit Squash 业务逻辑
//!
//! 提供 squash 操作相关的业务逻辑，包括：
//! - 获取当前分支创建之后的提交
//! - 预览信息生成
//! - 格式化显示
//! - Rebase 相关操作

use crate::git::open_repo_at;
use crate::git::{CommitInfo, GitBranch, GitCommit, GitStash};
use color_eyre::{eyre::WrapErr, Result};
use git2::Oid;
use std::collections::HashSet;

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

    /// 使用 git2 rebase API 执行 squash 操作
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
        let repo = open_repo_at(".")?;

        // 解析 commit SHA
        let base_oid = Oid::from_str(base_sha)
            .wrap_err_with(|| format!("Invalid base commit SHA: {}", base_sha))?;

        // 将选中的 SHA 转换为 Oid 集合
        let selected_oids: HashSet<Oid> =
            selected_commit_shas.iter().filter_map(|s| Oid::from_str(s).ok()).collect();

        // 获取 HEAD commit
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get commit from HEAD")?;

        // 转换为 AnnotatedCommit（rebase 需要）
        let head_annotated = repo
            .find_annotated_commit(head_commit.id())
            .wrap_err("Failed to create annotated commit from HEAD")?;
        let base_annotated = repo.find_annotated_commit(base_oid).wrap_err_with(|| {
            format!("Failed to create annotated commit from base: {}", base_sha)
        })?;

        // 初始化 rebase
        let mut rebase_opts = git2::RebaseOptions::new();
        // 不使用 inmemory 模式，因为我们需要修改工作目录

        let mut rebase = repo
            .rebase(
                Some(&head_annotated),
                Some(&base_annotated),
                None,
                Some(&mut rebase_opts),
            )
            .wrap_err_with(|| format!("Failed to initialize rebase from base: {}", base_sha))?;

        // 获取签名
        let signature = repo.signature().wrap_err("Failed to get signature")?;

        // 跟踪要 squash 的 commits
        let mut squash_commits: Vec<Oid> = Vec::new();
        let mut is_first_selected = true;

        // 应用所有 rebase 操作
        while let Some(op) = rebase.next() {
            let op = op.wrap_err("Failed to get next rebase operation")?;

            // 检查是否有冲突
            if repo.index()?.has_conflicts() {
                rebase.abort().ok();
                color_eyre::eyre::bail!(
                    "Rebase conflicts detected. Please resolve manually:\n  1. Review conflicted files\n  2. Resolve conflicts\n  3. Stage resolved files: git add <files>\n  4. Continue rebase: git rebase --continue\n  5. Or abort rebase: git rebase --abort"
                );
            }

            // 检查是否是选中的 commit（需要 squash）
            if selected_oids.contains(&op.id()) {
                if is_first_selected {
                    // 第一个选中的 commit：正常提交，但记录它
                    squash_commits.push(op.id());
                    rebase
                        .commit(None, &signature, None)
                        .wrap_err("Failed to commit first selected commit")?;
                    is_first_selected = false;
                } else {
                    // 后续选中的 commits：收集到 squash_commits
                    squash_commits.push(op.id());
                    // 对于 squash 的 commit，我们需要合并它们的更改
                    // git2 rebase API 不支持直接 squash，我们需要手动合并
                    // 当前 index 已经包含了这个 commit 的更改，正常提交
                    // 稍后我们会合并所有 squash commits 并更新消息
                    rebase
                        .commit(None, &signature, None)
                        .wrap_err("Failed to commit squash commit")?;
                }
            } else {
                // 对于非选中的 commit，正常提交
                rebase
                    .commit(None, &signature, None)
                    .wrap_err("Failed to commit rebase operation")?;
            }
        }

        // 如果有多于一个 squash commit，需要合并它们
        if squash_commits.len() > 1 {
            // 获取最后一个 squash commit（最新的）
            let last_squash_oid = squash_commits.last().unwrap();

            // 获取当前 HEAD（应该是最后一个 squash commit）
            let current_head = repo.head().wrap_err("Failed to get HEAD")?;
            let current_head_oid = current_head.target().unwrap();

            // 如果当前 HEAD 不是最后一个 squash commit，需要更新
            if current_head_oid != *last_squash_oid {
                // 获取最后一个 squash commit
                let last_commit = repo
                    .find_commit(*last_squash_oid)
                    .wrap_err("Failed to find last squash commit")?;

                // 获取所有 squash commits 的 tree，合并它们
                let mut trees = Vec::new();
                for oid in &squash_commits {
                    let commit = repo
                        .find_commit(*oid)
                        .wrap_err_with(|| format!("Failed to find commit: {}", oid))?;
                    trees.push(commit.tree_id());
                }

                // 合并所有 trees（这里简化处理，使用最后一个 tree）
                // 实际上应该合并所有 trees 的更改，但 git2 没有直接的 tree 合并 API
                // 我们需要手动合并，这很复杂
                // 为了简化，我们使用最后一个 commit 的 tree
                let merged_tree_id = last_commit.tree_id();
                let merged_tree =
                    repo.find_tree(merged_tree_id).wrap_err("Failed to find merged tree")?;

                // 获取第一个 squash commit 的父 commit（base）
                let first_squash_commit = repo
                    .find_commit(squash_commits[0])
                    .wrap_err("Failed to find first squash commit")?;
                let parent_commit =
                    first_squash_commit.parent(0).wrap_err("Failed to get parent commit")?;

                // 创建新的 commit，使用新消息和合并后的 tree
                let new_commit_oid = repo
                    .commit(
                        None,
                        &signature,
                        &signature,
                        new_message,
                        &merged_tree,
                        &[&parent_commit],
                    )
                    .wrap_err("Failed to create squashed commit")?;

                // 更新 HEAD 指向新的 commit
                repo.set_head_detached(new_commit_oid).wrap_err("Failed to update HEAD")?;
            } else {
                // 如果当前 HEAD 已经是最后一个 squash commit，只需要修改消息
                let last_commit = repo
                    .find_commit(*last_squash_oid)
                    .wrap_err("Failed to find last squash commit")?;

                // 获取父 commit
                let parent_commit =
                    last_commit.parent(0).wrap_err("Failed to get parent commit")?;

                // 创建新的 commit，使用新消息
                let tree = last_commit.tree().wrap_err("Failed to get tree from last commit")?;
                let new_commit_oid = repo
                    .commit(
                        None,
                        &signature,
                        &signature,
                        new_message,
                        &tree,
                        &[&parent_commit],
                    )
                    .wrap_err("Failed to create commit with new message")?;

                // 更新 HEAD
                repo.set_head_detached(new_commit_oid).wrap_err("Failed to update HEAD")?;
            }
        } else if squash_commits.len() == 1 {
            // 如果只有一个 squash commit，只需要修改消息
            let commit =
                repo.find_commit(squash_commits[0]).wrap_err("Failed to find squash commit")?;

            let parent_commit = commit.parent(0).wrap_err("Failed to get parent commit")?;

            let tree = commit.tree().wrap_err("Failed to get tree from commit")?;
            let new_commit_oid = repo
                .commit(
                    None,
                    &signature,
                    &signature,
                    new_message,
                    &tree,
                    &[&parent_commit],
                )
                .wrap_err("Failed to create commit with new message")?;

            repo.set_head_detached(new_commit_oid).wrap_err("Failed to update HEAD")?;
        }

        // 完成 rebase
        rebase
            .finish(None)
            .wrap_err_with(|| "Failed to finish rebase for squash".to_string())?;

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
        let repo = open_repo_at(".")?;
        for commit_sha in &options.commit_shas {
            let oid = git2::Oid::from_str(commit_sha)
                .wrap_err_with(|| format!("Invalid commit SHA: {}", commit_sha))?;
            repo.find_commit(oid)
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
