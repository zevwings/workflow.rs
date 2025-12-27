//! Git Cherry-pick 操作
//!
//! 本模块提供了 Git cherry-pick 相关的完整功能，包括：
//! - 应用提交到当前分支
//! - 应用提交但不提交（保留在工作区）
//! - 继续或中止 cherry-pick 操作
//! - 检查 cherry-pick 操作状态

use color_eyre::{eyre::WrapErr, Result};
use std::str::FromStr;

use super::helpers::open_repo;

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
        let repo = open_repo()?;

        // 解析提交哈希
        let commit_oid = git2::Oid::from_str(commit)
            .wrap_err_with(|| format!("Invalid commit SHA: {}", commit))?;

        // 获取要 cherry-pick 的提交
        let cherry_commit = repo
            .find_commit(commit_oid)
            .wrap_err_with(|| format!("Commit '{}' not found", commit))?;

        // 获取当前 HEAD
        let head = repo.head()?.peel_to_commit()?;

        // 获取提交的父提交（cherry-pick 的基础）
        let parent = cherry_commit.parent(0).ok();
        let parent_tree = parent.as_ref().map(|p| p.tree().ok()).flatten();

        // 获取要应用的树
        let cherry_tree = cherry_commit.tree()?;
        let head_tree = head.tree()?;

        // 合并树（cherry-pick 本质上是三路合并：parent -> cherry_commit 应用到 head）
        let mut merge_result = if let Some(parent_tree) = parent_tree {
            repo.merge_trees(&parent_tree, &head_tree, &cherry_tree, None)?
        } else {
            // 如果没有父提交（根提交），直接合并两个树
            repo.merge_trees(&head_tree, &head_tree, &cherry_tree, None)?
        };

        // 检查是否有冲突
        if merge_result.has_conflicts() {
            // 写入索引以便用户解决冲突
            merge_result.write()?;
            // 创建 CHERRY_PICK_HEAD 文件以标记正在进行 cherry-pick
            let git_dir = repo.path();
            std::fs::write(
                git_dir.join("CHERRY_PICK_HEAD"),
                commit_oid.to_string(),
            )?;
            color_eyre::eyre::bail!(
                "Cherry-pick conflict detected. Please resolve conflicts manually and continue."
            );
        }

        // 写入合并后的树到索引
        let tree_id = merge_result.write_tree_to(&repo)?;
        let tree = repo.find_tree(tree_id)?;

        // 创建新提交
        let signature = repo.signature()?;
        let message = cherry_commit
            .message()
            .unwrap_or("Cherry-pick commit");

        repo.commit(
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
        let repo = open_repo()?;

        // 解析提交哈希
        let commit_oid = git2::Oid::from_str(commit)
            .wrap_err_with(|| format!("Invalid commit SHA: {}", commit))?;

        // 获取要 cherry-pick 的提交
        let cherry_commit = repo
            .find_commit(commit_oid)
            .wrap_err_with(|| format!("Commit '{}' not found", commit))?;

        // 获取当前 HEAD
        let head = repo.head()?.peel_to_commit()?;

        // 获取提交的父提交（cherry-pick 的基础）
        let parent = cherry_commit.parent(0).ok();
        let parent_tree = parent.as_ref().map(|p| p.tree().ok()).flatten();

        // 获取要应用的树
        let cherry_tree = cherry_commit.tree()?;
        let head_tree = head.tree()?;

        // 合并树（cherry-pick 本质上是三路合并：parent -> cherry_commit 应用到 head）
        let mut merge_result = if let Some(parent_tree) = parent_tree {
            repo.merge_trees(&parent_tree, &head_tree, &cherry_tree, None)?
        } else {
            // 如果没有父提交（根提交），直接合并两个树
            repo.merge_trees(&head_tree, &head_tree, &cherry_tree, None)?
        };

        // 检查是否有冲突
        if merge_result.has_conflicts() {
            // 写入索引以便用户解决冲突
            merge_result.write()?;
            // 创建 CHERRY_PICK_HEAD 文件以标记正在进行 cherry-pick
            let git_dir = repo.path();
            std::fs::write(
                git_dir.join("CHERRY_PICK_HEAD"),
                commit_oid.to_string(),
            )?;
            color_eyre::eyre::bail!(
                "Cherry-pick conflict detected. Please resolve conflicts manually and continue."
            );
        }

        // 写入合并后的树到索引（但不提交）
        merge_result.write()?;

        // 将索引内容 checkout 到工作区
        let tree_id = merge_result.write_tree_to(&repo)?;
        let tree = repo.find_tree(tree_id)?;
        repo.checkout_tree(
            &tree.into_object(),
            Some(
                git2::build::CheckoutBuilder::default()
                    .force()
                    .remove_ignored(false)
                    .remove_untracked(false),
            ),
        )?;

        // 创建 CHERRY_PICK_HEAD 文件以标记正在进行 cherry-pick（no-commit 模式）
        let git_dir = repo.path();
        std::fs::write(
            git_dir.join("CHERRY_PICK_HEAD"),
            commit_oid.to_string(),
        )?;

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
        let repo = open_repo()?;

        // 检查是否正在进行 cherry-pick
        let git_dir = repo.path();
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

        // 获取要应用的提交
        let cherry_commit = repo.find_commit(commit_oid)?;

        // 获取当前 HEAD
        let head = repo.head()?.peel_to_commit()?;

        // 写入树并创建提交
        let tree_id = index.write_tree_to(&repo)?;
        let tree = repo.find_tree(tree_id)?;

        let signature = repo.signature()?;
        let message = cherry_commit
            .message()
            .unwrap_or("Cherry-pick commit");

        repo.commit(
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
        let repo = open_repo()?;

        // 检查是否正在进行 cherry-pick
        let git_dir = repo.path();
        let cherry_pick_head_path = git_dir.join("CHERRY_PICK_HEAD");
        if !cherry_pick_head_path.exists() {
            color_eyre::eyre::bail!("No cherry-pick in progress");
        }

        // 重置索引和工作区到 HEAD
        let head = repo.head()?.peel_to_commit()?;

        // 重置索引
        repo.reset(
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
        open_repo()
            .ok()
            .and_then(|repo| {
                let git_dir = repo.path();
                Some(git_dir.join("CHERRY_PICK_HEAD").exists())
            })
            .unwrap_or(false)
    }
}
