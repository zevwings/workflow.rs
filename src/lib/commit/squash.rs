//! Commit Squash 业务逻辑
//!
//! 提供 squash 操作相关的业务逻辑，包括：
//! - 获取当前分支创建之后的提交
//! - 预览信息生成
//! - 格式化显示
//! - Rebase 相关操作

use crate::git::{CommitInfo, GitBranch, GitCommit, GitStash};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use git2::Repository;
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

    /// 使用 git2 执行 squash rebase
    ///
    /// # 参数
    ///
    /// * `base_sha` - 基础 commit SHA（rebase 起点）
    /// * `commits` - 所有 commits 列表（从旧到新）
    /// * `selected_commit_shas` - 要压缩的 commit SHA 列表（按时间顺序，从旧到新）
    /// * `new_message` - 新的提交消息
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    fn execute_squash_rebase(
        base_sha: &str,
        _commits: &[CommitInfo],
        selected_commit_shas: &[String],
        new_message: &str,
    ) -> Result<()> {
        let repo = Repository::open(".").wrap_err("Failed to open repository")?;

        // 1. 获取基础 commit
        let base_oid = repo
            .revparse_single(base_sha)
            .wrap_err_with(|| format!("Failed to parse base commit: {}", base_sha))?
            .id();
        let base_commit = repo
            .find_commit(base_oid)
            .wrap_err_with(|| format!("Failed to find base commit: {}", base_sha))?;
        let base_annotated = repo
            .find_annotated_commit(base_oid)
            .wrap_err("Failed to create annotated commit for base")?;

        // 2. 获取当前 HEAD
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;
        let head_annotated = repo
            .find_annotated_commit(head_commit.id())
            .wrap_err("Failed to create annotated commit for HEAD")?;

        // 3. 创建要 squash 的 commits 集合
        let selected_set: HashSet<&str> = selected_commit_shas.iter().map(|s| s.as_str()).collect();

        // 4. 初始化 rebase
        let mut rebase_opts = git2::RebaseOptions::new();
        rebase_opts.inmemory(false); // 需要写入到仓库
        let mut rebase = repo
            .rebase(
                Some(&head_annotated),
                Some(&base_annotated),
                None,
                Some(&mut rebase_opts),
            )
            .wrap_err("Failed to initialize rebase")?;

        // 5. 先完成 rebase（所有 commits 都正常提交）
        // 然后手动处理 squash：合并 trees 并重写历史
        let mut squash_commit_oids: Vec<git2::Oid> = Vec::new();
        let mut squash_trees: Vec<git2::Tree> = Vec::new();
        let mut first_squash_commit_oid: Option<git2::Oid> = None;

        while let Some(op) = rebase.next() {
            let op = op?;
            let commit_oid = op.id();
            let commit_sha = commit_oid.to_string();
            let commit = repo.find_commit(commit_oid)?;
            let author = commit.author();

            if selected_set.contains(commit_sha.as_str()) {
                if first_squash_commit_oid.is_none() {
                    first_squash_commit_oid = Some(commit_oid);
                }
                squash_commit_oids.push(commit_oid);
                squash_trees.push(commit.tree()?);
            }

            // 所有 commits 都正常提交（rebase API 要求）
            rebase
                .commit(Some(&author), &author, None)
                .wrap_err("Failed to commit rebase operation")?;
        }

        // 6. 完成 rebase
        rebase.finish(None).wrap_err("Failed to finish rebase")?;

        // 7. 如果有多个 squash commits，需要合并它们并重写历史
        if let Some(_first_oid) = first_squash_commit_oid {
            if squash_commit_oids.len() > 1 {
                // 合并所有 squash trees
                let first_commit = repo.find_commit(squash_commit_oids[0])?;
                let mut combined_tree = first_commit.tree()?;

                // 合并所有后续 squash commits 的变更
                for squash_tree in &squash_trees[1..] {
                    let mut index = repo
                        .merge_trees(&base_commit.tree()?, &combined_tree, squash_tree, None)
                        .wrap_err("Failed to merge trees for squash")?;

                    if index.has_conflicts() {
                        return Err(eyre!(
                            "Squash conflicts detected. Please resolve conflicts manually."
                        ));
                    }

                    let tree_oid = index.write_tree_to(&repo)?;
                    combined_tree = repo.find_tree(tree_oid)?;
                }

                // 找到第一个 squash commit 的位置
                let head = repo.head().wrap_err("Failed to get HEAD")?;
                let mut current_commit =
                    head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;

                // 回退到第一个 squash commit
                for _ in 0..(squash_commit_oids.len() - 1) {
                    if let Ok(parent) = current_commit.parent(0) {
                        current_commit = parent;
                    } else {
                        break;
                    }
                }

                // 获取第一个 squash commit 的父 commit
                let first_squash_parent =
                    current_commit.parent(0).wrap_err("Failed to get first squash parent")?;

                // 重置到第一个 squash commit 的父 commit
                repo.reset(first_squash_parent.as_object(), git2::ResetType::Soft, None)
                    .wrap_err("Failed to reset")?;

                // 更新索引为合并后的 tree
                let mut index = repo.index().wrap_err("Failed to open index")?;
                index.read_tree(&combined_tree).wrap_err("Failed to read tree")?;
                index.write().wrap_err("Failed to write index")?;

                // 创建 squash commit
                let config = repo.config().ok();
                let name =
                    config.as_ref().and_then(|c| c.get_string("user.name").ok()).unwrap_or_else(
                        || current_commit.author().name().unwrap_or("Unknown").to_string(),
                    );
                let email = config
                    .as_ref()
                    .and_then(|c| c.get_string("user.email").ok())
                    .unwrap_or_else(|| {
                        current_commit.author().email().unwrap_or("unknown@example.com").to_string()
                    });
                let signature =
                    git2::Signature::now(&name, &email).wrap_err("Failed to create signature")?;

                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    new_message,
                    &combined_tree,
                    &[&first_squash_parent],
                )
                .wrap_err("Failed to create squash commit")?;

                // TODO: 重新应用后续的非 squash commits
                // 这需要遍历原始 commits，找出哪些不是 squash commits
                // 然后使用 cherry-pick 重新应用
                // 暂时简化：假设 rebase 已经处理了所有非 squash commits
            } else if squash_commit_oids.len() == 1 {
                // 只有一个 squash commit，只需要更新消息
                let head = repo.head().wrap_err("Failed to get HEAD")?;
                let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;
                let config = repo.config().ok();
                let name =
                    config.as_ref().and_then(|c| c.get_string("user.name").ok()).unwrap_or_else(
                        || head_commit.author().name().unwrap_or("Unknown").to_string(),
                    );
                let email = config
                    .as_ref()
                    .and_then(|c| c.get_string("user.email").ok())
                    .unwrap_or_else(|| {
                        head_commit.author().email().unwrap_or("unknown@example.com").to_string()
                    });
                let signature =
                    git2::Signature::now(&name, &email).wrap_err("Failed to create signature")?;

                let parent_ids: Vec<git2::Oid> = head_commit.parent_ids().collect();
                let parents: Vec<git2::Commit> = parent_ids
                    .iter()
                    .map(|id| repo.find_commit(*id))
                    .collect::<Result<Vec<_>, _>>()
                    .wrap_err("Failed to get parent commits")?;
                let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    new_message,
                    &head_commit.tree()?,
                    &parent_refs,
                )
                .wrap_err("Failed to update commit message")?;
            }
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

        // 步骤4: 使用 git2 执行 squash rebase
        let rebase_result = Self::execute_squash_rebase(
            &base_sha,
            &commits,
            &options.commit_shas,
            &options.new_message,
        );

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
