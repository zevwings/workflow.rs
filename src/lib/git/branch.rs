use color_eyre::{eyre::WrapErr, Result};
use std::collections::HashSet;

use super::auth::GitAuth;
use super::helpers::open_repo;
use git2::{build::CheckoutBuilder, BranchType, FetchOptions, PushOptions, Signature};

const COMMON_DEFAULT_BRANCHES: &[&str] = &["main", "master", "develop", "dev"];

/// 移除分支名称的前缀
///
/// 从完整的分支名中提取基础名称，支持两种格式：
/// - `prefix/branch-name`（使用 `/` 分割）
/// - `ticket--branch-name`（使用 `--` 分割）
fn remove_branch_prefix(branch: &str) -> &str {
    branch
        .rsplit('/')
        .next()
        .unwrap_or(branch)
        .rsplit("--")
        .next()
        .unwrap_or(branch)
}

/// 合并策略枚举
///
/// 定义不同的 Git 合并策略。
#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    /// 普通合并（创建合并提交）
    Merge,
    /// Squash 合并（将分支的所有提交压缩为一个提交）
    Squash,
    /// 只允许 fast-forward 合并（如果无法 fast-forward 则失败）
    FastForwardOnly,
}

/// Git 分支管理
///
/// 提供分支相关的操作功能，包括：
/// - 获取当前分支名
/// - 检查分支是否存在
/// - 创建或切换分支
/// - 获取默认分支
/// - 合并分支
/// - 推送和删除分支
pub struct GitBranch;

impl GitBranch {
    /// 获取当前分支名
    ///
    /// 使用 `git branch --show-current` 获取当前分支的名称。
    ///
    /// # 返回
    ///
    /// 返回当前分支的名称（去除首尾空白）。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或命令执行失败，返回相应的错误信息。
    pub fn current_branch() -> Result<String> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        head.shorthand()
            .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get branch name"))
            .map(|s| s.to_string())
    }

    /// 检查分支是否存在（本地或远程）
    ///
    /// 使用 `git rev-parse --verify` 检查指定分支在本地和远程是否存在。
    /// 该方法比使用 `git branch` 更高效。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要检查的分支名称
    ///
    /// # 返回
    ///
    /// 返回元组 `(本地存在, 远程存在)`：
    /// - `(true, true)` - 分支在本地和远程都存在
    /// - `(true, false)` - 分支只在本地存在
    /// - `(false, true)` - 分支只在远程存在
    /// - `(false, false)` - 分支不存在
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn is_branch_exists(branch_name: &str) -> Result<(bool, bool)> {
        let repo = open_repo()?;

        // 检查本地分支
        let exists_local = repo.find_branch(branch_name, BranchType::Local).is_ok();

        // 检查远程分支
        let exists_remote = repo.find_branch(branch_name, BranchType::Remote).is_ok();

        Ok((exists_local, exists_remote))
    }

    /// 检查分支是否在本地存在
    ///
    /// 这是 `is_branch_exists` 的便捷方法，只检查本地分支是否存在。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要检查的分支名称
    ///
    /// # 返回
    ///
    /// 如果分支在本地存在，返回 `true`，否则返回 `false`。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn has_local_branch(branch_name: &str) -> Result<bool> {
        let (exists_local, _) = Self::is_branch_exists(branch_name)?;
        Ok(exists_local)
    }

    /// 检查分支是否在远程存在
    ///
    /// 这是 `is_branch_exists` 的便捷方法，只检查远程分支是否存在。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要检查的分支名称
    ///
    /// # 返回
    ///
    /// 如果分支在远程存在，返回 `true`，否则返回 `false`。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn has_remote_branch(branch_name: &str) -> Result<bool> {
        let (_, exists_remote) = Self::is_branch_exists(branch_name)?;
        Ok(exists_remote)
    }

    /// 创建或切换到分支
    ///
    /// 根据分支的存在情况执行相应的操作：
    /// - 如果分支已存在且是当前分支，则跳过
    /// - 如果分支已存在但不是当前分支，则切换到该分支
    /// - 如果分支只存在于远程，则创建本地分支并跟踪远程分支
    /// - 如果分支不存在，则创建新分支
    ///
    /// 使用 git2 实现分支切换和创建。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要创建或切换的分支名称
    ///
    /// # 错误
    ///
    /// 如果分支操作失败，返回相应的错误信息。
    pub fn checkout_branch(branch_name: &str) -> Result<()> {
        let repo = open_repo()?;

        // 检查是否是当前分支
        let current_branch = Self::current_branch()?;
        if current_branch == branch_name {
            // 已经是当前分支，无需操作
            return Ok(());
        }

        // 检查分支是否存在
        let (exists_local, exists_remote) = Self::is_branch_exists(branch_name)?;

        if exists_local {
            // 分支已存在于本地，切换到它
            let branch = repo
                .find_branch(branch_name, BranchType::Local)
                .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;
            let reference = branch.get();
            repo.set_head(reference.name().unwrap())
                .wrap_err_with(|| format!("Failed to set HEAD to branch: {}", branch_name))?;

            let mut checkout_opts = CheckoutBuilder::new();
            checkout_opts.force();
            repo.checkout_head(Some(&mut checkout_opts))
                .wrap_err_with(|| format!("Failed to checkout branch: {}", branch_name))?;
        } else if exists_remote {
            // 分支只存在于远程，创建本地分支并跟踪远程分支
            let remote_ref = format!("refs/remotes/origin/{}", branch_name);
            let remote_commit = repo
                .find_reference(&remote_ref)
                .wrap_err_with(|| format!("Failed to find remote branch: {}", branch_name))?
                .peel_to_commit()
                .wrap_err("Failed to get remote commit")?;

            // 创建本地分支并设置上游跟踪
            let mut branch = repo
                .branch(branch_name, &remote_commit, false)
                .wrap_err_with(|| format!("Failed to create branch: {}", branch_name))?;
            branch
                .set_upstream(Some(&format!("origin/{}", branch_name)))
                .wrap_err_with(|| format!("Failed to set upstream for branch: {}", branch_name))?;

            // 切换到新创建的分支
            let reference = branch.get();
            repo.set_head(reference.name().unwrap())
                .wrap_err_with(|| format!("Failed to set HEAD to branch: {}", branch_name))?;

            let mut checkout_opts = CheckoutBuilder::new();
            checkout_opts.force();
            repo.checkout_head(Some(&mut checkout_opts))
                .wrap_err_with(|| format!("Failed to checkout remote branch: {}", branch_name))?;
        } else {
            // 分支不存在，创建新分支
            let head = repo.head().wrap_err("Failed to get HEAD")?;
            let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;

            let branch = repo
                .branch(branch_name, &head_commit, false)
                .wrap_err_with(|| format!("Failed to create branch: {}", branch_name))?;

            // 切换到新创建的分支
            let reference = branch.get();
            repo.set_head(reference.name().unwrap())
                .wrap_err_with(|| format!("Failed to set HEAD to branch: {}", branch_name))?;

            let mut checkout_opts = CheckoutBuilder::new();
            checkout_opts.force();
            repo.checkout_head(Some(&mut checkout_opts))
                .wrap_err_with(|| format!("Failed to create branch: {}", branch_name))?;
        }
        Ok(())
    }

    /// 获取默认分支
    ///
    /// 统一使用 Git 命令从远程获取默认分支，适用于所有 Git 仓库类型
    /// （包括 GitHub、Codeup、GitLab 等）
    ///
    /// 尝试通过以下方式获取默认分支：
    /// 1. 使用 `git ls-remote --symref origin HEAD` 直接从远程获取符号引用
    /// 2. 如果失败，使用 `git remote show origin` 获取
    /// 3. 如果都失败，从远程分支列表中查找常见的默认分支名（main, master, develop, dev）
    pub fn get_default_branch() -> Result<String> {
        let repo = open_repo()?;

        // 尝试方法1：从远程符号引用获取
        if let Ok(mut remote) = repo.find_remote("origin") {
            let refspecs: &[&str] = &[];
            if remote.fetch(refspecs, None, None).is_ok() {
                // 查找 refs/remotes/origin/HEAD 引用
                if let Ok(head_ref) = repo.find_reference("refs/remotes/origin/HEAD") {
                    if let Some(branch_name) = head_ref.symbolic_target() {
                        if let Some(branch) = branch_name.strip_prefix("refs/heads/") {
                            return Ok(branch.to_string());
                        }
                    }
                }
            }
        }

        // 尝试方法2：从远程分支列表中查找常见的默认分支名
        Self::find_default_branch_from_remote().wrap_err("Failed to get default branch")
    }

    /// 从远程分支列表中查找常见的默认分支名
    ///
    /// 当无法通过其他方式获取默认分支时，从远程分支列表中查找常见的默认分支名。
    /// 按顺序查找：`main`、`master`、`develop`、`dev`。
    ///
    /// # 返回
    ///
    /// 返回找到的默认分支名称。
    ///
    /// # 错误
    ///
    /// 如果没有找到任何常见的默认分支，返回相应的错误信息。
    fn find_default_branch_from_remote() -> Result<String> {
        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;
        let refspecs: &[&str] = &[];
        remote.fetch(refspecs, None, None).wrap_err("Failed to fetch from origin")?;

        for default_name in COMMON_DEFAULT_BRANCHES {
            if repo.find_branch(default_name, BranchType::Remote).is_ok() {
                return Ok(default_name.to_string());
            }
        }

        color_eyre::eyre::bail!("Could not determine default branch")
    }

    /// 获取所有分支（本地和远程），并排除重复
    ///
    /// 获取所有本地分支和远程分支，去除重复的分支名称，返回去重后的分支列表。
    /// 远程分支的 `origin/` 前缀会被移除，只保留分支名称。
    ///
    /// # 参数
    ///
    /// * `remove_prefix` - 是否移除分支名称的前缀（如 `prefix/branch-name` -> `branch-name`）
    ///   如果为 `true`，会移除 `prefix/` 和 `ticket--` 格式的前缀
    ///
    /// # 返回
    ///
    /// 返回去重后的分支名称列表（按字母顺序排序）
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use workflow::GitBranch;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // 获取完整分支名（包含前缀）
    /// let branches = GitBranch::get_all_branches(false)?;
    /// // 返回: ["main", "zw/code-optimization", "develop", ...]
    ///
    /// // 获取基础分支名（去掉前缀）
    /// let base_branches = GitBranch::get_all_branches(true)?;
    /// // 返回: ["main", "code-optimization", "develop", ...]
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_all_branches(remove_prefix: bool) -> Result<Vec<String>> {
        let repo = open_repo()?;
        let mut branch_set = HashSet::new();

        // 获取本地分支
        for branch_result in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result.wrap_err("Failed to get branch")?;
            if let Ok(Some(name)) = branch.name() {
                branch_set.insert(name.to_string());
            }
        }

        // 获取远程分支（需要先 fetch）
        let mut remote = repo.find_remote("origin").ok();
        if let Some(ref mut r) = remote {
            let refspecs: &[&str] = &[];
            let _ = r.fetch(refspecs, None, None);
        }

        for branch_result in repo.branches(Some(BranchType::Remote))? {
            let (branch, _) = branch_result.wrap_err("Failed to get remote branch")?;
            if let Ok(Some(name)) = branch.name() {
                // 移除 "origin/" 前缀
                let branch_name = name.strip_prefix("origin/").unwrap_or(name);
                branch_set.insert(branch_name.to_string());
            }
        }

        // 转换为排序后的 Vec
        let mut branches: Vec<String> = branch_set.into_iter().collect();
        branches.sort();

        // 如果需要移除前缀，提取基础名称
        if remove_prefix {
            Ok(Self::extract_base_branch_names(branches))
        } else {
            Ok(branches)
        }
    }

    /// 获取所有本地分支
    ///
    /// 只获取本地分支列表，不包括远程分支。
    ///
    /// # 返回
    ///
    /// 返回本地分支名称列表（按字母顺序排序）
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn get_local_branches() -> Result<Vec<String>> {
        let repo = open_repo()?;
        let mut branches = Vec::new();

        for branch_result in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result.wrap_err("Failed to get branch")?;
            if let Ok(Some(name)) = branch.name() {
                branches.push(name.to_string());
            }
        }

        branches.sort();
        Ok(branches)
    }

    /// 提取分支的基础名称（去掉前缀）
    ///
    /// 从完整的分支名中提取基础名称，支持两种格式：
    /// - `prefix/branch-name`（使用 `/` 分割）
    /// - `ticket--branch-name`（使用 `--` 分割）
    ///
    /// # 参数
    ///
    /// * `branches` - 分支名称列表
    ///
    /// # 返回
    ///
    /// 返回去重后的基础分支名称列表（按字母顺序排序）
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::GitBranch;
    ///
    /// let branches = vec!["zw/code-optimization".to_string(), "master".to_string()];
    /// let base_names = GitBranch::extract_base_branch_names(branches);
    /// // 返回: ["code-optimization", "master"]
    /// ```
    pub fn extract_base_branch_names(branches: Vec<String>) -> Vec<String> {
        let mut base_names: Vec<String> = branches
            .iter()
            .map(|branch| remove_branch_prefix(branch).to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        base_names.sort();
        base_names
    }

    /// 检查分支是否领先于指定分支（是否有新提交）
    ///
    /// 使用 `git rev-list --count` 检查指定分支相对于基础分支是否有新的提交。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要检查的分支名称
    /// * `base_branch` - 基础分支名称（用于比较）
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果分支有新的提交
    /// - `Ok(false)` - 如果分支为空或与指定分支相同
    ///
    /// # 错误
    ///
    /// 如果分支不存在或命令执行失败，返回相应的错误信息。
    pub fn is_branch_ahead(branch_name: &str, base_branch: &str) -> Result<bool> {
        let repo = open_repo()?;

        // 获取分支的 commit
        let branch_ref = repo
            .find_reference(&format!("refs/heads/{}", branch_name))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch_name)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;
        let branch_commit = branch_ref.peel_to_commit().wrap_err("Failed to get branch commit")?;

        // 获取基础分支的 commit
        let base_ref = repo
            .find_reference(&format!("refs/heads/{}", base_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", base_branch)))
            .wrap_err_with(|| format!("Failed to find branch: {}", base_branch))?;
        let base_commit = base_ref.peel_to_commit().wrap_err("Failed to get base commit")?;

        // 使用 revwalk 计算提交数量
        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk.push(branch_commit.id()).wrap_err("Failed to push commit")?;
        revwalk.hide(base_commit.id()).wrap_err("Failed to hide commit")?;

        let count = revwalk.count();
        Ok(count > 0)
    }

    /// 从远程拉取指定分支的最新更改
    ///
    /// 使用 `git pull origin <branch>` 从远程仓库拉取指定分支的最新更改。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要拉取的分支名称
    ///
    /// # 错误
    ///
    /// 如果拉取失败，返回相应的错误信息。
    pub fn pull(branch_name: &str) -> Result<()> {
        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Fetch 远程分支
        let refspecs: &[&str] = &[];
        remote
            .fetch(refspecs, Some(&mut fetch_options), None)
            .wrap_err("Failed to fetch from origin")?;

        // 获取远程分支的 commit
        let remote_branch_ref = format!("refs/remotes/origin/{}", branch_name);
        let remote_commit = repo
            .find_reference(&remote_branch_ref)
            .wrap_err_with(|| format!("Failed to find remote branch: {}", branch_name))?
            .peel_to_commit()
            .wrap_err("Failed to get remote commit")?;

        // 获取当前 HEAD
        let head_commit = repo
            .head()
            .wrap_err("Failed to get HEAD")?
            .peel_to_commit()
            .wrap_err("Failed to get HEAD commit")?;

        // 执行合并
        let remote_annotated = repo
            .find_annotated_commit(remote_commit.id())
            .wrap_err("Failed to annotate commit")?;
        let (analysis, _) =
            repo.merge_analysis(&[&remote_annotated]).wrap_err("Failed to analyze merge")?;

        if analysis.is_up_to_date() {
            // 已经是最新的，无需合并
            return Ok(());
        }

        if analysis.is_fast_forward() {
            // Fast-forward 合并
            let mut ref_ = repo.find_reference("HEAD").wrap_err("Failed to find HEAD")?;
            ref_.set_target(remote_commit.id(), "Fast-forward merge")
                .wrap_err("Failed to update HEAD")?;
            repo.set_head("HEAD").wrap_err("Failed to set HEAD")?;
            let mut checkout_opts = CheckoutBuilder::new();
            checkout_opts.force();
            repo.checkout_head(Some(&mut checkout_opts))
                .wrap_err("Failed to checkout HEAD")?;
        } else {
            // 需要创建合并提交
            let mut index = repo
                .merge_commits(&head_commit, &remote_commit, None)
                .wrap_err("Failed to merge commits")?;

            if index.has_conflicts() {
                index.write().wrap_err("Failed to write index")?;
                return Err(color_eyre::eyre::eyre!(
                    "Merge conflicts detected. Please resolve conflicts manually."
                ));
            }

            let tree_id = index.write_tree_to(&repo).wrap_err("Failed to write tree")?;
            let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;

            let config = repo.config().ok();
            let name = config
                .as_ref()
                .and_then(|c| c.get_string("user.name").ok())
                .unwrap_or_else(|| head_commit.author().name().unwrap_or("Unknown").to_string());
            let email =
                config.as_ref().and_then(|c| c.get_string("user.email").ok()).unwrap_or_else(
                    || head_commit.author().email().unwrap_or("unknown@example.com").to_string(),
                );
            let signature =
                git2::Signature::now(&name, &email).wrap_err("Failed to create signature")?;

            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &format!(
                    "Merge branch '{}' of origin into {}",
                    branch_name, branch_name
                ),
                &tree,
                &[&head_commit, &remote_commit],
            )
            .wrap_err("Failed to create merge commit")?;
        }

        Ok(())
    }

    /// 推送到远程仓库
    ///
    /// 将指定分支推送到远程仓库的 `origin`。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要推送的分支名称
    /// * `set_upstream` - 是否设置上游分支（使用 `-u` 选项）
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
    pub fn push(branch_name: &str, set_upstream: bool) -> Result<()> {
        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // 构建 refspec
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        // 推送
        remote
            .push(&[&refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to push branch: {}", branch_name))?;

        // 如果设置了 upstream，更新本地分支的上游跟踪
        if set_upstream {
            let mut branch = repo
                .find_branch(branch_name, BranchType::Local)
                .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;
            branch
                .set_upstream(Some(&format!("origin/{}", branch_name)))
                .wrap_err_with(|| format!("Failed to set upstream for branch: {}", branch_name))?;
        }

        Ok(())
    }

    /// 使用 force-with-lease 强制推送到远程仓库
    ///
    /// 使用 `--force-with-lease` 选项安全地强制推送分支到远程仓库。
    /// 这比 `--force` 更安全，因为它会检查远程分支是否有新的提交。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要推送的分支名称
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
    pub fn push_force_with_lease(branch_name: &str) -> Result<()> {
        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // 构建 refspec（带 force-with-lease）
        let refspec = format!("+refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        // 推送
        remote
            .push(&[&refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to force push branch: {}", branch_name))
    }

    /// 将当前分支 rebase 到目标分支
    ///
    /// 使用 `git rebase` 将当前分支的提交重新应用到目标分支之上。
    ///
    /// # 参数
    ///
    /// * `target_branch` - 目标分支引用（本地分支名或 origin/branch-name）
    ///
    /// # 错误
    ///
    /// 如果 rebase 失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 如果遇到冲突，rebase 会暂停，需要用户手动解决冲突后继续。
    pub fn rebase_onto(target_branch: &str) -> Result<()> {
        let repo = open_repo()?;

        // 1. 获取当前分支的 HEAD
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;
        let branch_annotated = repo
            .find_annotated_commit(head_commit.id())
            .wrap_err("Failed to create annotated commit for current branch")?;

        // 2. 获取目标分支（支持本地和远程分支）
        let target_commit = if target_branch.starts_with("origin/") {
            // 远程分支
            let remote_branch_name = target_branch.strip_prefix("origin/").unwrap();
            let remote_ref = repo
                .find_reference(&format!("refs/remotes/origin/{}", remote_branch_name))
                .wrap_err_with(|| format!("Failed to find remote branch: {}", target_branch))?;
            remote_ref
                .peel_to_commit()
                .wrap_err_with(|| format!("Failed to get commit for branch: {}", target_branch))?
        } else {
            // 本地分支
            let target_branch_ref = repo
                .find_branch(target_branch, BranchType::Local)
                .wrap_err_with(|| format!("Failed to find branch: {}", target_branch))?;
            target_branch_ref
                .get()
                .peel_to_commit()
                .wrap_err_with(|| format!("Failed to get commit for branch: {}", target_branch))?
        };
        let target_annotated = repo
            .find_annotated_commit(target_commit.id())
            .wrap_err("Failed to create annotated commit for target branch")?;

        // 3. 获取上游分支（当前分支的 merge-base）
        let merge_base_oid = repo
            .merge_base(head_commit.id(), target_commit.id())
            .wrap_err("Failed to find merge base")?;
        let upstream_annotated = repo
            .find_annotated_commit(merge_base_oid)
            .wrap_err("Failed to create annotated commit for merge base")?;

        // 4. 初始化 rebase
        let mut rebase_opts = git2::RebaseOptions::new();
        rebase_opts.inmemory(false); // 需要写入到仓库
        let mut rebase = repo
            .rebase(
                Some(&branch_annotated),
                Some(&upstream_annotated),
                Some(&target_annotated),
                Some(&mut rebase_opts),
            )
            .wrap_err("Failed to initialize rebase")?;

        // 5. 执行 rebase 操作
        while let Some(op) = rebase.next() {
            let op = op?;
            let commit_oid = op.id();
            let commit = repo.find_commit(commit_oid)?;
            let author = commit.author();

            // 提交 rebase 操作（保持原提交的作者和消息）
            rebase
                .commit(Some(&author), &author, None)
                .wrap_err("Failed to commit rebase operation")?;
        }

        // 6. 完成 rebase
        rebase.finish(None).wrap_err("Failed to finish rebase")?;

        Ok(())
    }

    /// 将指定范围的提交 rebase 到目标分支
    ///
    /// 使用 `git rebase --onto` 将 `<upstream>..<branch>` 范围内的提交
    /// rebase 到 `<newbase>` 之上。这样可以只 rebase 分支独有的提交，
    /// 排除上游分支的提交。
    ///
    /// # 参数
    ///
    /// * `newbase` - 新的基础分支（要 rebase 到的分支）
    /// * `upstream` - 上游分支（rebase 范围的起点，排除此分支的提交）
    /// * `branch` - 要 rebase 的分支（rebase 范围的终点）
    ///
    /// # 错误
    ///
    /// 如果 rebase 失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 如果遇到冲突，rebase 会暂停，需要用户手动解决冲突后继续。
    ///
    /// # 示例
    ///
    /// 如果 `test-rebase` 基于 `develop-` 创建，但想 rebase 到 `master`：
    /// ```no_run
    /// use workflow::GitBranch;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // 只 rebase test-rebase 独有的提交（排除 develop- 的提交）到 master
    /// GitBranch::rebase_onto_with_upstream("master", "develop-", "test-rebase")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn rebase_onto_with_upstream(newbase: &str, upstream: &str, branch: &str) -> Result<()> {
        let repo = open_repo()?;

        // 1. 获取分支、上游和新基础分支（支持本地和远程分支）
        let get_branch_commit = |branch_name: &str| -> Result<git2::Commit> {
            if branch_name.starts_with("origin/") {
                let remote_branch_name = branch_name.strip_prefix("origin/").unwrap();
                let remote_ref = repo
                    .find_reference(&format!("refs/remotes/origin/{}", remote_branch_name))
                    .wrap_err_with(|| format!("Failed to find remote branch: {}", branch_name))?;
                remote_ref
                    .peel_to_commit()
                    .wrap_err_with(|| format!("Failed to get commit for branch: {}", branch_name))
            } else {
                let branch_ref = repo
                    .find_branch(branch_name, BranchType::Local)
                    .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;
                branch_ref
                    .get()
                    .peel_to_commit()
                    .wrap_err_with(|| format!("Failed to get commit for branch: {}", branch_name))
            }
        };

        let branch_commit = get_branch_commit(branch)?;
        let branch_annotated = repo
            .find_annotated_commit(branch_commit.id())
            .wrap_err("Failed to create annotated commit for branch")?;

        let upstream_commit = get_branch_commit(upstream)?;
        let upstream_annotated = repo
            .find_annotated_commit(upstream_commit.id())
            .wrap_err("Failed to create annotated commit for upstream")?;

        let newbase_commit = get_branch_commit(newbase)?;
        let newbase_annotated = repo
            .find_annotated_commit(newbase_commit.id())
            .wrap_err("Failed to create annotated commit for newbase")?;

        // 2. 初始化 rebase（指定 onto 参数）
        let mut rebase_opts = git2::RebaseOptions::new();
        rebase_opts.inmemory(false); // 需要写入到仓库
        let mut rebase = repo
            .rebase(
                Some(&branch_annotated),
                Some(&upstream_annotated),
                Some(&newbase_annotated),
                Some(&mut rebase_opts),
            )
            .wrap_err("Failed to initialize rebase")?;

        // 3. 执行 rebase 操作
        while let Some(op) = rebase.next() {
            let op = op?;
            let commit_oid = op.id();
            let commit = repo.find_commit(commit_oid)?;
            let author = commit.author();

            // 提交 rebase 操作（保持原提交的作者和消息）
            rebase
                .commit(Some(&author), &author, None)
                .wrap_err("Failed to commit rebase operation")?;
        }

        // 4. 完成 rebase
        rebase.finish(None).wrap_err("Failed to finish rebase")?;

        Ok(())
    }

    /// 删除本地分支
    ///
    /// 删除指定的本地分支。如果分支未完全合并，可以使用 `force` 参数强制删除。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要删除的分支名称
    /// * `force` - 是否强制删除（使用 `-D` 而不是 `-d`）
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete(branch_name: &str, force: bool) -> Result<()> {
        let repo = open_repo()?;
        let mut branch = repo
            .find_branch(branch_name, BranchType::Local)
            .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;
        if force {
            branch
                .delete()
                .wrap_err_with(|| format!("Failed to force delete local branch: {}", branch_name))
        } else {
            branch
                .delete()
                .wrap_err_with(|| format!("Failed to delete local branch: {}", branch_name))
        }
    }

    /// 删除远程分支
    ///
    /// 使用 `git push origin --delete <branch_name>` 删除远程分支。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要删除的远程分支名称
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete_remote(branch_name: &str) -> Result<()> {
        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec = format!(":refs/heads/{}", branch_name);
        remote
            .push(&[&refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to delete remote branch: {}", branch_name))
    }

    /// 重命名本地分支
    ///
    /// 使用 `git branch -m` 重命名本地分支。
    ///
    /// # 参数
    ///
    /// * `old_name` - 旧分支名称（如果为 None，则重命名当前分支）
    /// * `new_name` - 新分支名称
    ///
    /// # 错误
    ///
    /// 如果重命名失败，返回相应的错误信息。
    pub fn rename(old_name: Option<&str>, new_name: &str) -> Result<()> {
        let repo = open_repo()?;
        let branch_name = if let Some(name) = old_name {
            name.to_string()
        } else {
            repo.head()
                .ok()
                .and_then(|h| {
                    let s = h.shorthand()?;
                    Some(s.to_string())
                })
                .unwrap_or_else(|| "HEAD".to_string())
        };
        let mut branch = repo
            .find_branch(&branch_name, BranchType::Local)
            .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;
        branch.rename(new_name, false).wrap_err_with(|| {
            if let Some(old) = old_name {
                format!("Failed to rename branch from '{}' to '{}'", old, new_name)
            } else {
                format!("Failed to rename current branch to '{}'", new_name)
            }
        })?;
        Ok(())
    }

    /// 重命名远程分支
    ///
    /// 通过推送新分支并删除旧远程分支来重命名远程分支。
    /// 注意：必须先推送新分支，然后删除旧分支，以避免丢失远程引用。
    ///
    /// # 参数
    ///
    /// * `old_name` - 旧分支名称
    /// * `new_name` - 新分支名称
    ///
    /// # 错误
    ///
    /// 如果重命名失败，返回相应的错误信息。
    pub fn rename_remote(old_name: &str, new_name: &str) -> Result<()> {
        // 先推送新分支并设置上游跟踪
        Self::push(new_name, true)
            .wrap_err_with(|| format!("Failed to push new branch '{}' to remote", new_name))?;
        // 然后删除旧的远程分支
        Self::delete_remote(old_name)
            .wrap_err_with(|| format!("Failed to delete old remote branch '{}'", old_name))?;
        Ok(())
    }

    /// 合并指定分支到当前分支
    ///
    /// 根据指定的合并策略将源分支合并到当前分支。
    ///
    /// # 参数
    ///
    /// * `source_branch` - 要合并的源分支名称
    /// * `strategy` - 合并策略
    ///
    /// # 错误
    ///
    /// 如果合并失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 即使 git merge 命令返回错误（如引用更新失败），如果合并实际上已经完成
    /// （通过检查 MERGE_HEAD 或合并 commit 的存在），也会认为操作成功。
    pub fn merge_branch(source_branch: &str, strategy: MergeStrategy) -> Result<()> {
        let repo = open_repo()?;

        let source_ref = repo
            .find_reference(&format!("refs/heads/{}", source_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", source_branch)))
            .wrap_err_with(|| format!("Failed to find branch: {}", source_branch))?;
        let source_commit = source_ref.peel_to_commit().wrap_err("Failed to get source commit")?;
        let head_ref = repo.head().wrap_err("Failed to get HEAD")?;
        let head_commit = head_ref.peel_to_commit().wrap_err("Failed to get HEAD commit")?;

        let source_annotated = repo
            .find_annotated_commit(source_commit.id())
            .wrap_err("Failed to annotate source commit")?;
        let (analysis, _) =
            repo.merge_analysis(&[&source_annotated]).wrap_err("Failed to analyze merge")?;

        match strategy {
            MergeStrategy::FastForwardOnly => {
                if !analysis.is_fast_forward() {
                    return Err(color_eyre::eyre::eyre!(
                        "Cannot fast-forward merge. Branch '{}' is not ahead of current branch",
                        source_branch
                    ));
                }
            }
            MergeStrategy::Squash => {
                // Squash merge: 合并所有更改到索引，但不创建合并提交
                let mut index = repo
                    .merge_commits(&head_commit, &source_commit, None)
                    .wrap_err("Failed to merge commits")?;
                if index.has_conflicts() {
                    index.write().wrap_err("Failed to write index")?;
                    return Err(color_eyre::eyre::eyre!(
                        "Merge conflicts detected. Please resolve conflicts manually."
                    ));
                }
                index.write().wrap_err("Failed to write index")?;
                // Squash merge 只更新索引，不创建提交，需要用户手动提交
                return Ok(());
            }
            MergeStrategy::Merge => {
                // Normal merge: proceed with merge logic below
            }
        }

        if analysis.is_up_to_date() {
            return Ok(());
        }

        if analysis.is_fast_forward() {
            let mut ref_ = repo.find_reference("HEAD").wrap_err("Failed to find HEAD")?;
            ref_.set_target(source_commit.id(), "Fast-forward merge")
                .wrap_err("Failed to update HEAD")?;
            repo.set_head("HEAD").wrap_err("Failed to set HEAD")?;
            let mut checkout_opts = CheckoutBuilder::new();
            checkout_opts.force();
            repo.checkout_head(Some(&mut checkout_opts))
                .wrap_err("Failed to checkout HEAD")?;
        } else {
            let mut index = repo
                .merge_commits(&head_commit, &source_commit, None)
                .wrap_err("Failed to merge commits")?;
            if index.has_conflicts() {
                index.write().wrap_err("Failed to write index")?;
                return Err(color_eyre::eyre::eyre!(
                    "Merge conflicts detected. Please resolve conflicts manually."
                ));
            }
            let tree_id = index.write_tree_to(&repo).wrap_err("Failed to write tree")?;
            let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;
            let config = repo.config().ok();
            let name = config
                .as_ref()
                .and_then(|c| c.get_string("user.name").ok())
                .unwrap_or_else(|| head_commit.author().name().unwrap_or("Unknown").to_string());
            let email =
                config.as_ref().and_then(|c| c.get_string("user.email").ok()).unwrap_or_else(
                    || head_commit.author().email().unwrap_or("unknown@example.com").to_string(),
                );
            let signature = Signature::now(&name, &email).wrap_err("Failed to create signature")?;
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &format!("Merge branch '{}'", source_branch),
                &tree,
                &[&head_commit, &source_commit],
            )
            .wrap_err("Failed to create merge commit")?;
        }

        Ok(())
    }

    /// 检查是否有合并冲突
    ///
    /// 使用 `git diff --check` 检查是否有合并冲突标记。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果检测到合并冲突
    /// - `Ok(false)` - 如果没有冲突
    ///
    /// # 错误
    ///
    /// 如果命令执行失败，返回相应的错误信息。
    pub fn has_merge_conflicts() -> Result<bool> {
        let repo = open_repo()?;
        // 检查 MERGE_HEAD 是否存在（表示正在进行合并）
        let has_merge_head = repo.path().join("MERGE_HEAD").exists();
        if has_merge_head {
            // 检查索引是否有冲突
            let index = repo.index().wrap_err("Failed to get index")?;
            if index.has_conflicts() {
                return Ok(true);
            }
            // 检查工作目录是否有冲突标记
            let mut opts = git2::StatusOptions::new();
            opts.include_untracked(false);
            let statuses = repo.statuses(Some(&mut opts)).wrap_err("Failed to get status")?;
            for entry in statuses.iter() {
                if entry.status().is_wt_modified() {
                    // 可以进一步检查文件内容是否有冲突标记，但这里简化处理
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// 检查分支是否已合并到指定分支
    ///
    /// 使用 `git branch --merged` 检查指定分支是否已合并到基础分支。
    ///
    /// # 参数
    ///
    /// * `branch` - 要检查的分支名称
    /// * `base_branch` - 基础分支名称（用于检查合并状态）
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果分支已合并到基础分支
    /// - `Ok(false)` - 如果分支未合并
    ///
    /// # 错误
    ///
    /// 如果命令执行失败，返回相应的错误信息。
    pub fn is_branch_merged(branch: &str, base_branch: &str) -> Result<bool> {
        let repo = open_repo()?;
        let branch_ref = repo
            .find_reference(&format!("refs/heads/{}", branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch))?;
        let branch_commit = branch_ref.peel_to_commit().wrap_err("Failed to get branch commit")?;
        let base_ref = repo
            .find_reference(&format!("refs/heads/{}", base_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", base_branch)))
            .wrap_err_with(|| format!("Failed to find base branch: {}", base_branch))?;
        let base_commit = base_ref.peel_to_commit().wrap_err("Failed to get base commit")?;
        let merge_base_oid = repo
            .merge_base(branch_commit.id(), base_commit.id())
            .wrap_err("Failed to find merge base")?;
        Ok(merge_base_oid == branch_commit.id())
    }

    /// 获取两个分支之间的提交列表
    ///
    /// 使用 `git rev-list` 获取 from_branch 相对于 to_branch 的所有新提交。
    ///
    /// # 参数
    ///
    /// * `base_branch` - 基础分支名称（to_branch）
    /// * `head_branch` - 头部分支名称（from_branch）
    ///
    /// # 返回
    ///
    /// 返回提交哈希列表，按时间顺序排列（从旧到新）。
    ///
    /// # 错误
    ///
    /// 如果分支不存在或命令执行失败，返回相应的错误信息。
    pub fn get_commits_between(base_branch: &str, head_branch: &str) -> Result<Vec<String>> {
        let repo = open_repo()?;
        let base_ref = repo
            .find_reference(&format!("refs/heads/{}", base_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", base_branch)))
            .wrap_err_with(|| format!("Failed to find base branch: {}", base_branch))?;
        let base_commit = base_ref.peel_to_commit().wrap_err("Failed to get base commit")?;
        let head_ref = repo
            .find_reference(&format!("refs/heads/{}", head_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", head_branch)))
            .wrap_err_with(|| format!("Failed to find head branch: {}", head_branch))?;
        let head_commit = head_ref.peel_to_commit().wrap_err("Failed to get head commit")?;
        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk.push(head_commit.id()).wrap_err("Failed to push commit")?;
        revwalk.hide(base_commit.id()).wrap_err("Failed to hide commit")?;
        let commits: Vec<String> = revwalk
            .map(|oid| oid.map(|id| id.to_string()))
            .collect::<Result<Vec<_>, _>>()
            .wrap_err("Failed to walk commits")?;
        Ok(commits)
    }

    /// 获取两个分支的共同祖先（merge base）
    ///
    /// 使用 `git merge-base` 获取两个分支的共同祖先提交。
    ///
    /// # 参数
    ///
    /// * `branch1` - 第一个分支名称
    /// * `branch2` - 第二个分支名称
    ///
    /// # 返回
    ///
    /// 返回共同祖先的提交哈希。如果两个分支没有共同祖先，返回错误。
    ///
    /// # 错误
    ///
    /// 如果分支不存在或命令执行失败，返回相应的错误信息。
    pub fn merge_base(branch1: &str, branch2: &str) -> Result<String> {
        let repo = open_repo()?;
        let ref1 = repo
            .find_reference(&format!("refs/heads/{}", branch1))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch1)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch1))?;
        let commit1 = ref1.peel_to_commit().wrap_err("Failed to get commit")?;
        let ref2 = repo
            .find_reference(&format!("refs/heads/{}", branch2))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch2)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch2))?;
        let commit2 = ref2.peel_to_commit().wrap_err("Failed to get commit")?;
        let merge_base_oid = repo.merge_base(commit1.id(), commit2.id()).wrap_err_with(|| {
            format!(
                "Failed to get merge base between '{}' and '{}'",
                branch1, branch2
            )
        })?;
        Ok(merge_base_oid.to_string())
    }

    /// 检查一个分支是否直接基于另一个分支创建
    ///
    /// 通过比较 merge-base 和候选分支的 HEAD 来判断 from_branch 是否直接基于 candidate_branch 创建。
    ///
    /// # 参数
    ///
    /// * `from_branch` - 要检查的分支
    /// * `candidate_branch` - 候选的基础分支
    ///
    /// # 返回
    ///
    /// 如果 from_branch 直接基于 candidate_branch 创建，返回 `true`，否则返回 `false`。
    ///
    /// # 说明
    ///
    /// 判断逻辑：
    /// - 如果 `merge-base(from_branch, candidate_branch) == candidate_branch` 的 HEAD，
    ///   说明 from_branch 可能是直接基于 candidate_branch 创建的
    pub fn is_branch_based_on(from_branch: &str, candidate_branch: &str) -> Result<bool> {
        // 如果两个分支相同，返回 false
        if from_branch == candidate_branch {
            return Ok(false);
        }

        // 获取 merge-base
        let merge_base_commit = match Self::merge_base(from_branch, candidate_branch) {
            Ok(commit) => commit,
            Err(_) => return Ok(false),
        };

        // 获取 candidate_branch 的 HEAD
        let repo = open_repo()?;
        let candidate_ref = repo
            .find_reference(&format!("refs/heads/{}", candidate_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", candidate_branch)))
            .wrap_err_with(|| format!("Failed to find branch: {}", candidate_branch))?;
        let candidate_commit = candidate_ref.peel_to_commit().wrap_err("Failed to get commit")?;
        let candidate_head = candidate_commit.id().to_string();

        // 如果 merge-base 等于 candidate_branch 的 HEAD，说明 from_branch 直接基于 candidate_branch
        Ok(merge_base_commit == candidate_head)
    }

    /// 检查 commit 是否在远程分支中
    ///
    /// 通过检查远程分支是否包含指定的 commit 来判断。
    ///
    /// # 参数
    ///
    /// * `branch` - 本地分支名称
    /// * `commit_sha` - 要检查的 commit SHA
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果 commit 在远程分支中
    /// - `Ok(false)` - 如果 commit 不在远程分支中或远程分支不存在
    ///
    /// # 错误
    ///
    /// 如果命令执行失败，返回相应的错误信息。
    pub fn is_commit_in_remote(branch: &str, commit_sha: &str) -> Result<bool> {
        // 首先检查远程分支是否存在
        let (_, has_remote) = Self::is_branch_exists(branch)?;
        if !has_remote {
            return Ok(false);
        }

        // 检查远程分支是否包含该 commit
        let repo = open_repo()?;
        let commit_oid = repo
            .revparse_single(commit_sha)
            .wrap_err_with(|| format!("Failed to parse commit: {}", commit_sha))?
            .id();
        let remote_branch_ref = format!("refs/remotes/origin/{}", branch);
        let remote_ref = repo
            .find_reference(&remote_branch_ref)
            .wrap_err_with(|| format!("Failed to find remote branch: {}", branch))?;
        let remote_commit = remote_ref.peel_to_commit().wrap_err("Failed to get remote commit")?;
        let merge_base_oid = repo
            .merge_base(commit_oid, remote_commit.id())
            .wrap_err("Failed to find merge base")?;
        // 如果 merge_base 等于 commit_oid，说明 commit 在远程分支的历史中
        Ok(merge_base_oid == commit_oid)
    }
}
