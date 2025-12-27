//! Git Cherry-pick 操作
//!
//! 本模块提供了 Git cherry-pick 相关的完整功能，包括：
//! - 应用提交到当前分支
//! - 应用提交但不提交（保留在工作区）
//! - 继续或中止 cherry-pick 操作
//! - 检查 cherry-pick 操作状态

use color_eyre::{eyre::WrapErr, Result};

use super::GitRepository;

/// Git Cherry-pick 管理
///
/// 提供 cherry-pick 相关的操作功能，包括：
/// - 应用提交到当前分支
/// - 应用提交但不提交（保留在工作区）
/// - 继续或中止 cherry-pick 操作
/// - 检查 cherry-pick 操作状态
pub struct GitCherryPick;

impl GitCherryPick {
    /// Cherry-pick 提交到当前分支
    ///
    /// 使用 git2 库将指定的提交应用到当前分支。
    ///
    /// # 参数
    ///
    /// * `commit` - 要 cherry-pick 的提交哈希
    ///
    /// # 错误
    ///
    /// 如果 cherry-pick 失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 如果遇到冲突，cherry-pick 会暂停，需要用户手动解决冲突后继续。
    pub fn cherry_pick(commit: &str) -> Result<()> {
        let mut repo = GitRepository::open()?;

        // 解析提交哈希
        let commit_oid = git2::Oid::from_str(commit)
            .wrap_err_with(|| format!("Invalid commit SHA: {}", commit))?;

        // 获取当前 HEAD OID（在获取可变引用之前）
        let head_oid = repo
            .head()?
            .target()
            .ok_or_else(|| color_eyre::eyre::eyre!("HEAD reference does not point to a commit"))?;

        // 获取要 cherry-pick 的提交和当前 HEAD（使用可变引用）
        let repo_inner = repo.as_inner_mut();
        let cherry_commit = repo_inner
            .find_commit(commit_oid)
            .wrap_err_with(|| format!("Commit '{}' not found", commit))?;
        let head = repo_inner.find_commit(head_oid).wrap_err("Failed to get HEAD commit")?;

        // 获取提交的父提交（cherry-pick 的基础）
        let parent = cherry_commit.parent(0).ok();
        let parent_tree = parent.as_ref().and_then(|p| p.tree().ok());

        // 获取要应用的树
        let cherry_tree = cherry_commit.tree()?;
        let head_tree = head.tree()?;

        // 合并树（cherry-pick 本质上是三路合并：parent -> cherry_commit 应用到 head）
        let mut merge_result = if let Some(parent_tree) = parent_tree {
            repo_inner.merge_trees(&parent_tree, &head_tree, &cherry_tree, None)?
        } else {
            // 如果没有父提交（根提交），直接合并两个树
            repo_inner.merge_trees(&head_tree, &head_tree, &cherry_tree, None)?
        };

        // 检查是否有冲突
        if merge_result.has_conflicts() {
            // 写入索引以便用户解决冲突
            merge_result.write()?;
            // 创建 CHERRY_PICK_HEAD 文件以标记正在进行 cherry-pick
            let git_dir = repo_inner.path();
            std::fs::write(git_dir.join("CHERRY_PICK_HEAD"), commit_oid.to_string())?;
            color_eyre::eyre::bail!(
                "Cherry-pick conflict detected. Please resolve conflicts manually and continue."
            );
        }

        // 写入合并后的树到索引
        let tree_id = merge_result.write_tree_to(repo_inner)?;
        let tree = repo_inner.find_tree(tree_id)?;

        // 创建新提交
        let signature = repo_inner.signature().wrap_err("Failed to get repository signature")?;
        let message = cherry_commit.message().unwrap_or("Cherry-pick commit");

        repo_inner.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&head],
        )?;

        Ok(())
    }

    /// Cherry-pick 提交到当前分支（不提交）
    ///
    /// 使用 git2 库将指定的提交应用到当前分支的工作区，
    /// 但不创建提交。修改会保留在工作区（未暂存状态）。
    ///
    /// # 参数
    ///
    /// * `commit` - 要 cherry-pick 的提交哈希
    ///
    /// # 错误
    ///
    /// 如果 cherry-pick 失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// - 如果遇到冲突，cherry-pick 会暂停，需要用户手动解决冲突后继续
    /// - 修改会保留在工作区，需要手动提交
    pub fn cherry_pick_no_commit(commit: &str) -> Result<()> {
        let mut repo = GitRepository::open()?;

        // 解析提交哈希
        let commit_oid = git2::Oid::from_str(commit)
            .wrap_err_with(|| format!("Invalid commit SHA: {}", commit))?;

        // 获取当前 HEAD OID（在获取可变引用之前）
        let head_oid = repo
            .head()?
            .target()
            .ok_or_else(|| color_eyre::eyre::eyre!("HEAD reference does not point to a commit"))?;

        // 获取要 cherry-pick 的提交和当前 HEAD（使用可变引用）
        let repo_inner = repo.as_inner_mut();
        let cherry_commit = repo_inner
            .find_commit(commit_oid)
            .wrap_err_with(|| format!("Commit '{}' not found", commit))?;
        let head = repo_inner.find_commit(head_oid).wrap_err("Failed to get HEAD commit")?;

        // 获取提交的父提交（cherry-pick 的基础）
        let parent = cherry_commit.parent(0).ok();
        let parent_tree = parent.as_ref().and_then(|p| p.tree().ok());

        // 获取要应用的树
        let cherry_tree = cherry_commit.tree()?;
        let head_tree = head.tree()?;

        // 合并树（cherry-pick 本质上是三路合并：parent -> cherry_commit 应用到 head）
        let mut merge_result = if let Some(parent_tree) = parent_tree {
            repo_inner.merge_trees(&parent_tree, &head_tree, &cherry_tree, None)?
        } else {
            // 如果没有父提交（根提交），直接合并两个树
            repo_inner.merge_trees(&head_tree, &head_tree, &cherry_tree, None)?
        };

        // 检查是否有冲突
        if merge_result.has_conflicts() {
            // 写入索引以便用户解决冲突
            merge_result.write()?;
            // 创建 CHERRY_PICK_HEAD 文件以标记正在进行 cherry-pick
            let git_dir = repo_inner.path();
            std::fs::write(git_dir.join("CHERRY_PICK_HEAD"), commit_oid.to_string())?;
            color_eyre::eyre::bail!(
                "Cherry-pick conflict detected. Please resolve conflicts manually and continue."
            );
        }

        // 写入合并后的树到索引（但不提交）
        merge_result.write()?;

        // 将索引内容 checkout 到工作区
        let tree_id = merge_result.write_tree_to(repo_inner)?;
        let tree = repo_inner.find_tree(tree_id)?;
        repo_inner.checkout_tree(
            &tree.into_object(),
            Some(
                git2::build::CheckoutBuilder::default()
                    .force()
                    .remove_ignored(false)
                    .remove_untracked(false),
            ),
        )?;

        // 创建 CHERRY_PICK_HEAD 文件以标记正在进行 cherry-pick（no-commit 模式）
        let git_dir = repo_inner.path();
        std::fs::write(git_dir.join("CHERRY_PICK_HEAD"), commit_oid.to_string())?;

        Ok(())
    }

    /// 继续 cherry-pick 操作
    ///
    /// 在解决冲突后，使用 git2 库继续 cherry-pick 操作。
    ///
    /// # 错误
    ///
    /// 如果继续操作失败，返回相应的错误信息。
    pub fn cherry_pick_continue() -> Result<()> {
        let mut repo = GitRepository::open()?;

        // 检查是否正在进行 cherry-pick
        let git_dir = repo.as_inner().path();
        let cherry_pick_head_path = git_dir.join("CHERRY_PICK_HEAD");
        if !cherry_pick_head_path.exists() {
            color_eyre::eyre::bail!("No cherry-pick in progress");
        }

        // 读取 CHERRY_PICK_HEAD 获取要应用的提交
        let commit_sha = std::fs::read_to_string(&cherry_pick_head_path)
            .wrap_err("Failed to read CHERRY_PICK_HEAD")?;
        let commit_oid = git2::Oid::from_str(commit_sha.trim())
            .wrap_err("Invalid commit SHA in CHERRY_PICK_HEAD")?;

        // 检查是否有未解决的冲突
        let mut index = repo.index()?;
        if index.has_conflicts() {
            color_eyre::eyre::bail!(
                "Cherry-pick conflicts not resolved. Please resolve conflicts before continuing."
            );
        }

        // 获取当前 HEAD OID（在获取可变引用之前）
        let head_oid = repo
            .head()?
            .target()
            .ok_or_else(|| color_eyre::eyre::eyre!("HEAD reference does not point to a commit"))?;

        // 获取要应用的提交和当前 HEAD（使用可变引用）
        let repo_inner = repo.as_inner_mut();
        let cherry_commit = repo_inner.find_commit(commit_oid)?;
        let head = repo_inner.find_commit(head_oid).wrap_err("Failed to get HEAD commit")?;

        // 写入树并创建提交
        let tree_id = index.write_tree_to(repo_inner)?;
        let tree = repo_inner.find_tree(tree_id)?;

        let signature = repo_inner.signature().wrap_err("Failed to get repository signature")?;
        let message = cherry_commit.message().unwrap_or("Cherry-pick commit");

        repo_inner.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&head],
        )?;

        // 删除 CHERRY_PICK_HEAD 文件
        std::fs::remove_file(&cherry_pick_head_path)
            .wrap_err("Failed to remove CHERRY_PICK_HEAD")?;

        Ok(())
    }

    /// 中止 cherry-pick 操作
    ///
    /// 使用 git2 库中止当前的 cherry-pick 操作。
    ///
    /// # 错误
    ///
    /// 如果中止操作失败，返回相应的错误信息。
    pub fn cherry_pick_abort() -> Result<()> {
        let mut repo = GitRepository::open()?;

        // 检查是否正在进行 cherry-pick
        let git_dir = repo.as_inner().path();
        let cherry_pick_head_path = git_dir.join("CHERRY_PICK_HEAD");
        if !cherry_pick_head_path.exists() {
            color_eyre::eyre::bail!("No cherry-pick in progress");
        }

        // 重置索引和工作区到 HEAD
        let head_oid = repo
            .head()?
            .target()
            .ok_or_else(|| color_eyre::eyre::eyre!("HEAD reference does not point to a commit"))?;

        // 获取 HEAD commit（使用可变引用）
        let repo_inner = repo.as_inner_mut();
        let head = repo_inner.find_commit(head_oid).wrap_err("Failed to get HEAD commit")?;

        // 重置索引
        repo_inner.reset(
            &head.into_object(),
            git2::ResetType::Mixed,
            Some(
                git2::build::CheckoutBuilder::default()
                    .force()
                    .remove_ignored(false)
                    .remove_untracked(false),
            ),
        )?;

        // 删除 CHERRY_PICK_HEAD 文件
        std::fs::remove_file(&cherry_pick_head_path)
            .wrap_err("Failed to remove CHERRY_PICK_HEAD")?;

        Ok(())
    }

    /// 检查是否正在进行 cherry-pick 操作
    ///
    /// 使用 git2 库通过检查 `.git/CHERRY_PICK_HEAD` 文件是否存在来判断。
    ///
    /// # 返回
    ///
    /// 如果正在进行 cherry-pick 操作，返回 `true`，否则返回 `false`。
    pub fn is_cherry_pick_in_progress() -> bool {
        GitRepository::open()
            .ok()
            .map(|repo| {
                let git_dir = repo.as_inner().path();
                git_dir.join("CHERRY_PICK_HEAD").exists()
            })
            .unwrap_or(false)
    }
}
