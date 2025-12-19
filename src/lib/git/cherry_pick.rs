//! Git Cherry-pick 操作
//!
//! 本模块提供了 Git cherry-pick 相关的完整功能，包括：
//! - 应用提交到当前分支
//! - 应用提交但不提交（保留在工作区）
//! - 继续或中止 cherry-pick 操作
//! - 检查 cherry-pick 操作状态

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use git2::{build::CheckoutBuilder, Signature};
use std::fs;
use std::path::Path;

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
    /// 使用 git2 将指定的提交应用到当前分支。
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

        // 1. 解析提交引用
        let commit_oid = repo
            .revparse_single(commit)
            .wrap_err_with(|| format!("Failed to parse commit: {}", commit))?
            .id();
        let commit_to_pick = repo
            .find_commit(commit_oid)
            .wrap_err_with(|| format!("Failed to find commit: {}", commit))?;

        // 2. 获取当前 HEAD
        let head_commit = repo
            .head()
            .wrap_err("Failed to get HEAD")?
            .peel_to_commit()
            .wrap_err("Failed to get HEAD commit")?;

        // 3. 执行三向合并
        let mut index = repo
            .merge_commits(&head_commit, &commit_to_pick, None)
            .wrap_err("Failed to merge commits")?;

        // 4. 检查冲突
        if index.has_conflicts() {
            index.write().wrap_err("Failed to write index")?;
            // 创建 CHERRY_PICK_HEAD 标记正在进行 cherry-pick
            let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
            fs::write(&cherry_pick_head, commit_oid.to_string())
                .wrap_err("Failed to create CHERRY_PICK_HEAD")?;
            return Err(eyre!(
                "Cherry-pick conflict detected. Please resolve conflicts manually."
            ));
        }

        // 5. 写入索引
        index.write().wrap_err("Failed to write index")?;

        // 6. 创建提交
        let tree_id = index.write_tree_to(&repo).wrap_err("Failed to write tree")?;
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;

        let config = repo.config().ok();
        let name = config
            .as_ref()
            .and_then(|c| c.get_string("user.name").ok())
            .unwrap_or_else(|| commit_to_pick.author().name().unwrap_or("Unknown").to_string());
        let email =
            config
                .as_ref()
                .and_then(|c| c.get_string("user.email").ok())
                .unwrap_or_else(|| {
                    commit_to_pick.author().email().unwrap_or("unknown@example.com").to_string()
                });
        let signature = Signature::now(&name, &email).wrap_err("Failed to create signature")?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            commit_to_pick.message().unwrap_or("Cherry-pick commit"),
            &tree,
            &[&head_commit],
        )
        .wrap_err("Failed to create commit")?;

        Ok(())
    }

    /// Cherry-pick 提交到当前分支（不提交）
    ///
    /// 使用 git2 将指定的提交应用到当前分支的工作区，
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

        // 1. 解析提交引用
        let commit_oid = repo
            .revparse_single(commit)
            .wrap_err_with(|| format!("Failed to parse commit: {}", commit))?
            .id();
        let commit_to_pick = repo
            .find_commit(commit_oid)
            .wrap_err_with(|| format!("Failed to find commit: {}", commit))?;

        // 2. 获取当前 HEAD
        let head_commit = repo
            .head()
            .wrap_err("Failed to get HEAD")?
            .peel_to_commit()
            .wrap_err("Failed to get HEAD commit")?;

        // 3. 执行三向合并
        let mut index = repo
            .merge_commits(&head_commit, &commit_to_pick, None)
            .wrap_err("Failed to merge commits")?;

        // 4. 检查冲突
        if index.has_conflicts() {
            index.write().wrap_err("Failed to write index")?;
            // 创建 CHERRY_PICK_HEAD 标记正在进行 cherry-pick
            let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
            fs::write(&cherry_pick_head, commit_oid.to_string())
                .wrap_err("Failed to create CHERRY_PICK_HEAD")?;
            return Err(eyre!(
                "Cherry-pick conflict detected. Please resolve conflicts manually."
            ));
        }

        // 5. 写入索引
        index.write().wrap_err("Failed to write index")?;

        // 6. Checkout 到工作区（应用更改但不提交）
        let tree_id = index.write_tree_to(&repo).wrap_err("Failed to write tree")?;
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;
        let mut checkout_opts = CheckoutBuilder::new();
        checkout_opts.force();
        repo.checkout_tree(tree.as_object(), Some(&mut checkout_opts))
            .wrap_err("Failed to checkout tree")?;

        // 7. 创建 CHERRY_PICK_HEAD（标记正在进行 cherry-pick）
        let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
        fs::write(&cherry_pick_head, commit_oid.to_string())
            .wrap_err("Failed to create CHERRY_PICK_HEAD")?;

        Ok(())
    }

    /// 继续 cherry-pick 操作
    ///
    /// 在解决冲突后，使用 git2 继续 cherry-pick 操作。
    ///
    /// # 错误
    ///
    /// 如果继续操作失败，返回相应的错误信息。
    pub fn cherry_pick_continue() -> Result<()> {
        let repo = open_repo()?;

        // 1. 检查是否有 CHERRY_PICK_HEAD
        let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
        if !cherry_pick_head.exists() {
            return Err(eyre!("No cherry-pick in progress"));
        }

        // 2. 读取要 cherry-pick 的提交 SHA
        let commit_sha =
            fs::read_to_string(&cherry_pick_head).wrap_err("Failed to read CHERRY_PICK_HEAD")?;
        let commit_oid =
            commit_sha.trim().parse::<git2::Oid>().wrap_err("Failed to parse commit SHA")?;
        let commit_to_pick = repo.find_commit(commit_oid).wrap_err("Failed to find commit")?;

        // 3. 检查是否还有冲突
        let mut index = repo.index().wrap_err("Failed to open index")?;
        if index.has_conflicts() {
            return Err(eyre!(
                "Conflicts still exist. Please resolve all conflicts before continuing."
            ));
        }

        // 4. 获取当前 HEAD
        let head_commit = repo
            .head()
            .wrap_err("Failed to get HEAD")?
            .peel_to_commit()
            .wrap_err("Failed to get HEAD commit")?;

        // 5. 写入 tree 并创建提交
        let tree_id = index.write_tree_to(&repo).wrap_err("Failed to write tree")?;
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;

        let config = repo.config().ok();
        let name = config
            .as_ref()
            .and_then(|c| c.get_string("user.name").ok())
            .unwrap_or_else(|| commit_to_pick.author().name().unwrap_or("Unknown").to_string());
        let email =
            config
                .as_ref()
                .and_then(|c| c.get_string("user.email").ok())
                .unwrap_or_else(|| {
                    commit_to_pick.author().email().unwrap_or("unknown@example.com").to_string()
                });
        let signature = Signature::now(&name, &email).wrap_err("Failed to create signature")?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            commit_to_pick.message().unwrap_or("Cherry-pick commit"),
            &tree,
            &[&head_commit],
        )
        .wrap_err("Failed to create commit")?;

        // 6. 删除 CHERRY_PICK_HEAD
        fs::remove_file(&cherry_pick_head).wrap_err("Failed to remove CHERRY_PICK_HEAD")?;

        Ok(())
    }

    /// 中止 cherry-pick 操作
    ///
    /// 使用 git2 中止当前的 cherry-pick 操作。
    ///
    /// # 错误
    ///
    /// 如果中止操作失败，返回相应的错误信息。
    pub fn cherry_pick_abort() -> Result<()> {
        let repo = open_repo()?;

        // 检查是否有 CHERRY_PICK_HEAD
        let cherry_pick_head = repo.path().join("CHERRY_PICK_HEAD");
        if !cherry_pick_head.exists() {
            return Err(eyre!("No cherry-pick in progress"));
        }

        // 清理状态并重置到 HEAD
        repo.cleanup_state().wrap_err("Failed to cleanup state")?;

        // 删除 CHERRY_PICK_HEAD
        fs::remove_file(&cherry_pick_head).wrap_err("Failed to remove CHERRY_PICK_HEAD")?;

        Ok(())
    }

    /// 检查是否正在进行 cherry-pick 操作
    ///
    /// 通过检查 `.git/CHERRY_PICK_HEAD` 文件是否存在来判断。
    ///
    /// # 返回
    ///
    /// 如果正在进行 cherry-pick 操作，返回 `true`，否则返回 `false`。
    pub fn is_cherry_pick_in_progress() -> bool {
        if let Ok(repo) = open_repo() {
            repo.path().join("CHERRY_PICK_HEAD").exists()
        } else {
            Path::new(".git/CHERRY_PICK_HEAD").exists()
        }
    }
}
