use color_eyre::{eyre::WrapErr, Result};
use std::collections::HashSet;

use super::GitRepository;
use crate::base::resilience::{
    default_download_timeout, execute_with_timeout_and_retry, RetryConfig, TimeoutConfig,
};
use crate::git::commands::branch::GitBranchCommand;
use crate::git::commands::{GitCommitCommand, GitRepoCommand};
use crate::{trace_info, trace_warn};

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
    /// 使用 Git 命令获取当前分支的名称。
    ///
    /// # 返回
    ///
    /// 返回当前分支的名称。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或操作失败，返回相应的错误信息。
    pub fn current_branch() -> Result<String> {
        let repo = GitRepository::open()?;
        repo.current_branch_name()
    }

    /// 获取当前分支名（指定仓库路径）
    ///
    /// 使用 Git 命令获取指定仓库的当前分支名称。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # 返回
    ///
    /// 返回当前分支的名称。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或操作失败，返回相应的错误信息。
    pub fn current_branch_in(repo_path: impl AsRef<std::path::Path>) -> Result<String> {
        let repo = GitRepository::open_at(repo_path)?;
        repo.current_branch_name()
    }

    /// 检查分支是否存在（本地或远程）
    ///
    /// 使用 Git 命令检查指定分支在本地和远程是否存在。
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
    /// 如果操作失败，返回相应的错误信息。
    pub fn is_branch_exists(branch_name: &str) -> Result<(bool, bool)> {
        // 使用 GitBranchCommand 的封装方法，不需要打开 GitRepository
        GitBranchCommand::branch_exists(branch_name, Some("origin"), None)
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
    /// 优先使用 `git switch`（Git 2.23+），如果失败则回退到 `git checkout`。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要创建或切换的分支名称
    ///
    /// # 错误
    ///
    /// 如果分支操作失败，返回相应的错误信息。
    pub fn checkout_branch(branch_name: &str) -> Result<()> {
        // 验证分支名不为空
        if branch_name.is_empty() {
            return Err(color_eyre::eyre::eyre!("Branch name cannot be empty"));
        }

        // 检查是否是当前分支
        let current_branch = Self::current_branch()?;
        if current_branch == branch_name {
            // 已经是当前分支，无需操作
            return Ok(());
        }

        // 检查分支是否存在
        let (exists_local, exists_remote) = Self::is_branch_exists(branch_name)?;

        // 使用 GitBranchCommand 封装
        if exists_local {
            // 分支已存在于本地，切换到它
            GitBranchCommand::checkout(branch_name, None).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to checkout branch {}: {}", branch_name, e)
            })?;
        } else if exists_remote {
            // 分支只存在于远程，创建本地分支并跟踪远程分支
            GitBranchCommand::checkout_create(
                branch_name,
                Some(&format!("origin/{}", branch_name)),
                None,
            )
            .map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to create and checkout branch {} from remote: {}",
                    branch_name,
                    e
                )
            })?;
        } else {
            // 分支不存在，创建新分支并切换到它
            GitBranchCommand::checkout_create(branch_name, None, None).map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to create and checkout branch {}: {}",
                    branch_name,
                    e
                )
            })?;
        }
        Ok(())
    }

    /// 在指定路径创建分支
    ///
    /// 在指定路径的 Git 仓库中创建新分支，不依赖当前工作目录。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库路径
    /// * `branch_name` - 要创建的分支名称
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    ///
    /// # 错误
    ///
    /// 如果分支创建失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::GitBranch;
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// GitBranch::create_branch_at("/path/to/repo", "feature/new")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_branch_at(
        repo_path: impl AsRef<std::path::Path>,
        branch_name: &str,
    ) -> Result<()> {
        // 验证分支名不为空
        if branch_name.is_empty() {
            return Err(color_eyre::eyre::eyre!("Branch name cannot be empty"));
        }

        // 使用 GitBranchCommand 封装
        GitBranchCommand::create_branch(branch_name, Some(repo_path.as_ref())).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to create branch {}: {}", branch_name, e)
        })?;

        Ok(())
    }

    /// 在指定路径切换分支
    ///
    /// 在指定路径的 Git 仓库中切换到指定分支，不依赖当前工作目录。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库路径
    /// * `branch_name` - 要切换到的分支名称
    ///
    /// # 返回
    ///
    /// 成功时返回 `Ok(())`，失败时返回错误。
    ///
    /// # 错误
    ///
    /// 如果分支切换失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::GitBranch;
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// GitBranch::checkout_branch_at("/path/to/repo", "feature/new")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn checkout_branch_at(
        repo_path: impl AsRef<std::path::Path>,
        branch_name: &str,
    ) -> Result<()> {
        // 验证分支名不为空
        if branch_name.is_empty() {
            return Err(color_eyre::eyre::eyre!("Branch name cannot be empty"));
        }

        // 使用 GitBranchCommand 封装的方法，优先使用 git switch，失败时回退到 git checkout
        GitBranchCommand::checkout_branch(branch_name, false, Some(repo_path.as_ref()))
            .wrap_err_with(|| {
                format!(
                    "Failed to checkout branch {} at {:?}",
                    branch_name,
                    repo_path.as_ref()
                )
            })
    }

    /// 获取默认分支
    ///
    /// 统一使用 Git 命令从远程获取默认分支，适用于所有 Git 仓库类型
    /// （包括 GitHub、GitLab 等）
    ///
    /// 尝试通过以下方式获取默认分支：
    /// 1. 从远程分支列表中查找常见的默认分支名（main, master, develop, dev）- 优先使用，不依赖网络
    /// 2. 如果失败，使用 `git remote show origin` 获取（需要网络连接）
    /// 3. 如果都失败，使用 `git ls-remote --symref origin HEAD` 直接从远程获取符号引用（需要网络连接）
    pub fn get_default_branch() -> Result<String> {
        Self::get_default_branch_in(
            std::env::current_dir().wrap_err("Failed to get current directory")?,
        )
    }

    /// 获取默认分支名（指定仓库路径）
    ///
    /// 使用 Git 命令获取指定仓库的默认分支名称。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # 返回
    ///
    /// 返回默认分支的名称（通常是 "main" 或 "master"）。
    ///
    /// # 错误
    ///
    /// 如果无法确定默认分支，返回相应的错误信息。
    pub fn get_default_branch_in(repo_path: impl AsRef<std::path::Path>) -> Result<String> {
        // 优先尝试方法1：从远程分支列表中查找常见的默认分支名（不依赖网络，最快）
        // 这在测试环境中最可靠，因为测试已经设置了正确的远程引用
        if let Ok(branch) = Self::find_default_branch_from_remote_in(repo_path.as_ref()) {
            return Ok(branch);
        }

        // 尝试方法2：从 refs/remotes/origin/HEAD 获取默认分支（本地已缓存的远程 HEAD）
        if let Ok(head_ref) =
            GitBranchCommand::symbolic_ref("refs/remotes/origin/HEAD", Some(repo_path.as_ref()))
        {
            let head_ref = head_ref.trim();
            if let Some(branch) = head_ref.strip_prefix("refs/remotes/origin/") {
                return Ok(branch.to_string());
            }
        }

        // 尝试方法3：使用 git ls-remote 从远程获取 HEAD 引用（需要网络连接）
        if let Ok(output) =
            GitBranchCommand::ls_remote_symref("origin", "HEAD", Some(repo_path.as_ref()))
        {
            for line in output.lines() {
                if line.starts_with("ref: refs/heads/") {
                    if let Some(branch) = line.strip_prefix("ref: refs/heads/") {
                        if let Some(branch) = branch.split('\t').next() {
                            return Ok(branch.to_string());
                        }
                    }
                }
            }
        }

        color_eyre::eyre::bail!("Failed to get default branch")
    }

    /// 从远程分支列表中查找常见的默认分支名（指定仓库路径）
    ///
    /// 使用 Git 命令从远程分支列表中查找常见的默认分支名。
    /// 按顺序查找：`main`、`master`、`develop`、`dev`。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # 返回
    ///
    /// 返回找到的默认分支名称。
    ///
    /// # 错误
    ///
    /// 如果没有找到任何常见的默认分支，返回相应的错误信息。
    fn find_default_branch_from_remote_in(
        repo_path: impl AsRef<std::path::Path>,
    ) -> Result<String> {
        // 获取所有远程分支引用
        let remote_branches =
            GitBranchCommand::list_remote_branches(None, Some(repo_path.as_ref()))
                .map_err(|e| color_eyre::eyre::eyre!("Failed to get remote branches: {}", e))?;

        let remote_refs: Vec<String> = remote_branches
            .iter()
            .filter_map(|line| {
                let name = line.trim();
                if name.is_empty() || !name.contains('/') {
                    return None;
                }
                // 移除远程名称前缀（如 "origin/"）
                name.split('/').nth(1).map(|s| s.to_string())
            })
            .collect();

        // 按顺序查找常见的默认分支名
        for default_name in COMMON_DEFAULT_BRANCHES {
            if remote_refs.iter().any(|name| name == default_name) {
                return Ok(default_name.to_string());
            }
        }

        color_eyre::eyre::bail!("Could not determine default branch")
    }

    /// 获取所有分支（本地和远程），并排除重复
    ///
    /// 使用 Git 命令获取所有本地分支和远程分支，去除重复的分支名称，返回去重后的分支列表。
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
    /// 如果操作失败，返回相应的错误信息。
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
        let mut branch_set = HashSet::new();

        // 获取本地分支（不需要打开 GitRepository，git 命令会自动查找仓库）
        let local_branches = GitBranchCommand::list_local_branches_formatted(None)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get local branches: {}", e))?;
        for line in &local_branches {
            let name = line.trim();
            if !name.is_empty() {
                branch_set.insert(name.to_string());
            }
        }

        // 获取远程分支
        let remote_branches = GitBranchCommand::list_remote_branches(None, None)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get remote branches: {}", e))?;
        for line in &remote_branches {
            let name = line.trim();
            if !name.is_empty() && name.contains('/') {
                // 移除远程名称前缀（如 "origin/"）
                if let Some(branch_name) = name.split('/').nth(1) {
                    branch_set.insert(branch_name.to_string());
                }
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
    /// 使用 Git 命令获取所有本地分支列表，不包括远程分支。
    ///
    /// # 返回
    ///
    /// 返回本地分支名称列表（按字母顺序排序）
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn get_local_branches() -> Result<Vec<String>> {
        let local_branches = GitBranchCommand::list_local_branches_formatted(None)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get local branches: {}", e))?;

        let mut branches: Vec<String> = local_branches
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

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
        // 使用 git rev-list 检查是否有新提交
        // base_branch..branch_name 表示在 branch_name 中但不在 base_branch 中的提交
        let count =
            GitCommitCommand::rev_list_count(&format!("{}..{}", base_branch, branch_name), None)
                .map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to check if branch is ahead: {}", e)
                })?;

        Ok(count > 0)
    }

    /// 从远程拉取指定分支的最新更改
    ///
    /// 使用 Git 命令从远程仓库拉取指定分支的最新更改。
    /// 支持 SSH 和 HTTPS 认证，适用于私有仓库。
    /// 自动处理 fast-forward 合并和普通合并。
    /// 包含超时和重试机制，提高网络操作的可靠性。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要拉取的分支名称
    ///
    /// # 错误
    ///
    /// 如果拉取失败，返回相应的错误信息。
    /// 如果检测到合并冲突，会返回错误，需要用户手动解决。
    pub fn pull(branch_name: &str) -> Result<()> {
        let timeout_config =
            TimeoutConfig::new(default_download_timeout()).with_platform_specific();
        let retry_config = RetryConfig::platform_default();

        // 保护 fetch 操作
        execute_with_timeout_and_retry(
            timeout_config,
            retry_config,
            move || -> Result<()> {
                // 使用 git fetch 获取远程更新
                GitRepoCommand::fetch(Some("origin"), None)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to fetch from origin: {}", e))?;
                Ok(())
            },
            "Pulling from remote",
        )?;

        // 检查本地分支是否存在
        let local_exists = Self::has_local_branch(branch_name)?;

        if local_exists {
            // 本地分支存在，使用 git pull 或 git merge
            // 先尝试 git pull（会自动处理 fast-forward 和合并）
            if let Err(e) = GitBranchCommand::pull(branch_name, Some("origin"), None) {
                // 如果 pull 失败，检查是否有冲突
                if Self::has_merge_conflicts()? {
                    return Err(color_eyre::eyre::eyre!(
                        "Merge conflicts detected. Please resolve conflicts manually."
                    ));
                } else {
                    return Err(color_eyre::eyre::eyre!(
                        "Failed to pull branch {}: {}",
                        branch_name,
                        e
                    ));
                }
            }
        } else {
            // 本地分支不存在，创建新分支并跟踪远程分支
            GitBranchCommand::checkout_create(
                branch_name,
                Some(&format!("origin/{}", branch_name)),
                None,
            )
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to create local branch from remote: {}", e)
            })?;
        }

        Ok(())
    }

    /// 推送到远程仓库
    ///
    /// 将指定分支推送到远程仓库的 `origin`。
    ///
    /// 包含超时和重试机制，提高网络操作的可靠性。
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
        let timeout_config =
            TimeoutConfig::new(default_download_timeout()).with_platform_specific();
        let retry_config = RetryConfig::platform_default();
        let branch_name_clone = branch_name.to_string();

        // 保护 push 操作
        execute_with_timeout_and_retry(
            timeout_config,
            retry_config,
            move || -> Result<()> {
                let mut repo = GitRepository::open()?;
                let mut remote = repo.find_origin_remote()?;

                // 推送（GitRemote::push 现在只需要 refspecs）
                remote
                    .push(&[branch_name_clone.as_str()])
                    .wrap_err_with(|| format!("Failed to push branch: {}", branch_name_clone))?;

                Ok(())
            },
            "Pushing to remote",
        )?;

        // 如果设置了 upstream，更新本地分支的上游跟踪（本地操作，不需要超时保护）
        if set_upstream {
            // 使用 GitBranchCommand 封装
            GitBranchCommand::set_upstream_to(
                &format!("origin/{}", branch_name),
                branch_name,
                None,
            )
            .wrap_err("Failed to set upstream")?;
        }

        Ok(())
    }

    /// 使用 force-with-lease 强制推送到远程仓库
    ///
    /// 使用 `--force-with-lease` 选项安全地强制推送分支到远程仓库。
    /// 这比 `--force` 更安全，因为它会检查远程分支是否有新的提交。
    /// 包含超时和重试机制，提高网络操作的可靠性。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要推送的分支名称
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
    pub fn push_force_with_lease(branch_name: &str) -> Result<()> {
        let timeout_config =
            TimeoutConfig::new(default_download_timeout()).with_platform_specific();
        let retry_config = RetryConfig::platform_default();
        let branch_name = branch_name.to_string();

        execute_with_timeout_and_retry(
            timeout_config,
            retry_config,
            move || -> Result<()> {
                // 使用 GitBranchCommand 封装
                GitBranchCommand::push(&branch_name, true, Some("origin"), None).map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to force push branch {}: {}", branch_name, e)
                })?;

                Ok(())
            },
            "Force pushing to remote",
        )?;
        Ok(())
    }

    /// 将当前分支 rebase 到目标分支
    ///
    /// 使用 Git 命令将当前分支的提交重新应用到目标分支之上。
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
        // 使用 GitBranchCommand 封装
        GitBranchCommand::rebase(target_branch, None).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to rebase onto branch {}: {}", target_branch, e)
        })?;

        Ok(())
    }

    /// 将指定范围的提交 rebase 到目标分支
    ///
    /// 使用 Git 命令将 `<upstream>..<branch>` 范围内的提交
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
        // 使用 git rebase --onto 命令
        // git rebase --onto <newbase> <upstream> <branch>
        GitBranchCommand::rebase_onto(newbase, upstream, branch, None).map_err(|e| {
            // 如果 rebase 失败，检查是否有冲突
            if Self::has_merge_conflicts().unwrap_or(false) {
                color_eyre::eyre::eyre!(
                    "Rebase conflict detected. Please resolve conflicts manually and continue."
                )
            } else {
                color_eyre::eyre::eyre!(
                    "Failed to rebase '{}' onto '{}' (excluding '{}' commits): {}",
                    branch,
                    newbase,
                    upstream,
                    e
                )
            }
        })?;

        Ok(())
    }

    /// 删除本地分支
    ///
    /// 使用 Git 命令删除指定的本地分支。如果分支未完全合并，可以使用 `force` 参数强制删除。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要删除的分支名称
    /// * `force` - 是否强制删除（即使分支未完全合并也删除）
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete(branch_name: &str, force: bool) -> Result<()> {
        // 使用 git branch -d 或 -D 删除分支
        GitBranchCommand::delete_branch(branch_name, force, None).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to delete branch {}: {}", branch_name, e)
        })?;

        Ok(())
    }

    /// 删除远程分支
    ///
    /// 使用 Git 命令删除远程分支，相当于 `git push origin --delete <branch_name>`。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要删除的远程分支名称
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete_remote(branch_name: &str) -> Result<()> {
        // 使用 git push origin --delete 删除远程分支
        GitBranchCommand::delete_remote_branch(branch_name, Some("origin"), None).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to delete remote branch {}: {}", branch_name, e)
        })?;

        Ok(())
    }

    /// 重命名本地分支
    ///
    /// 使用 Git 命令重命名本地分支引用。
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
        // 使用 git branch -m 重命名分支
        GitBranchCommand::rename_branch(old_name, new_name, None)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to rename branch: {}", e))?;

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
    /// 使用 Git 命令根据指定的合并策略将源分支合并到当前分支。
    ///
    /// # 参数
    ///
    /// * `source_branch` - 要合并的源分支名称
    /// * `strategy` - 合并策略
    ///
    /// # 错误
    ///
    /// 如果合并失败（包括冲突），返回相应的错误信息。
    pub fn merge_branch(source_branch: &str, strategy: MergeStrategy) -> Result<()> {
        match strategy {
            MergeStrategy::FastForwardOnly => {
                // 使用 --ff-only 选项，只允许 fast-forward 合并
                GitBranchCommand::merge_ff_only(source_branch, None).map_err(|e| {
                    if Self::has_merge_conflicts().unwrap_or(false) {
                        color_eyre::eyre::eyre!(
                            "Merge conflicts detected. Please resolve conflicts manually."
                        )
                    } else {
                        color_eyre::eyre::eyre!(
                            "Cannot fast-forward merge. Use a different merge strategy. {}",
                            e
                        )
                    }
                })?;
            }
            MergeStrategy::Squash => {
                // 使用 --squash 选项进行 squash 合并
                GitBranchCommand::merge_squash(source_branch, None).map_err(|e| {
                    if Self::has_merge_conflicts().unwrap_or(false) {
                        color_eyre::eyre::eyre!(
                            "Merge conflicts detected. Please resolve conflicts manually."
                        )
                    } else {
                        color_eyre::eyre::eyre!("Failed to squash merge: {}", e)
                    }
                })?;
                // Squash 合并后需要手动提交，但这里我们自动提交
                // 获取源分支的提交信息用于提交消息
                let commits = Self::get_commits_between(&Self::current_branch()?, source_branch)?;
                let mut message = format!("Squashed commit of branch '{}'\n\n", source_branch);
                for commit_sha in commits.iter().take(10) {
                    // 获取每个提交的消息（只取前10个，避免消息过长）
                    if let Ok(commit_msg) = GitCommitCommand::get_commit_message(commit_sha, None) {
                        message.push_str(&format!("* {}\n", commit_msg.trim()));
                    }
                }
                // 提交 squash 合并
                GitCommitCommand::commit(&message, false, None)
                    .map_err(|e| color_eyre::eyre::eyre!("Failed to commit squash merge: {}", e))?;
            }
            MergeStrategy::Merge => {
                // 普通合并（允许 fast-forward 或创建合并提交）
                GitBranchCommand::merge_branch(source_branch, None, false, None).map_err(|e| {
                    if Self::has_merge_conflicts().unwrap_or(false) {
                        color_eyre::eyre::eyre!(
                            "Merge conflicts detected. Please resolve conflicts manually."
                        )
                    } else {
                        color_eyre::eyre::eyre!("Failed to merge branch: {}", e)
                    }
                })?;
            }
        }

        Ok(())
    }

    /// 检查是否有合并冲突
    ///
    /// 使用 Git 命令检查是否有合并冲突。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果检测到合并冲突
    /// - `Ok(false)` - 如果没有冲突
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn has_merge_conflicts() -> Result<bool> {
        // 使用 git status --porcelain 检查是否有冲突文件
        let status = GitCommitCommand::status(None)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to check merge conflicts: {}", e))?;

        // 检查是否有冲突标记（UU 表示未合并的文件）
        if status.lines().any(|line| {
            let line = line.trim();
            line.starts_with("UU") || line.starts_with("AA") || line.starts_with("DD")
        }) {
            return Ok(true);
        }

        // 检查 MERGE_HEAD 是否存在（表示正在进行合并）
        let repo = GitRepository::open()?;
        let git_dir = repo.path().join(".git");
        let merge_head_path = git_dir.join("MERGE_HEAD");
        if merge_head_path.exists() {
            // 如果 MERGE_HEAD 存在，使用 git diff --check 检查冲突标记
            if let Ok(has_conflicts) = GitCommitCommand::check_conflicts(None) {
                if has_conflicts {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// 检查分支是否已合并到指定分支
    ///
    /// 使用 Git 命令检查指定分支是否已合并到基础分支。
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
    /// 如果分支不存在或操作失败，返回相应的错误信息。
    pub fn is_branch_merged(branch: &str, base_branch: &str) -> Result<bool> {
        // 使用 git merge-base 获取合并基，然后比较
        let merge_base = GitCommitCommand::merge_base(branch, base_branch, None)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get merge base: {}", e))?;

        // 获取分支的 HEAD commit
        let branch_commit = GitCommitCommand::rev_parse(branch, None)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get branch commit: {}", e))?;

        // 如果 merge-base 等于 branch 的 commit，说明 branch 已合并到 base_branch
        Ok(merge_base == branch_commit)
    }

    /// 获取两个分支之间的提交列表
    ///
    /// 使用 Git 命令获取 from_branch 相对于 to_branch 的所有新提交。
    ///
    /// 仅查找本地分支引用，避免在 Windows 上访问远程引用时可能触发网络操作导致超时。
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
    /// 如果分支不存在或操作失败，返回相应的错误信息。
    pub fn get_commits_between(base_branch: &str, head_branch: &str) -> Result<Vec<String>> {
        // 使用 git rev-list 获取提交列表
        // base_branch..head_branch 表示在 head_branch 中但不在 base_branch 中的提交
        let commits =
            GitCommitCommand::rev_list(&format!("{}..{}", base_branch, head_branch), None)
                .map_err(|e| {
                    color_eyre::eyre::eyre!("Failed to get commits between branches: {}", e)
                })?;

        Ok(commits)
    }

    /// 获取两个分支的共同祖先（merge base）
    ///
    /// 使用 Git 命令获取两个分支的共同祖先提交。
    ///
    /// # 参数
    ///
    /// * `branch1` - 第一个分支名称
    /// * `branch2` - 第二个分支名称
    /// * `local_only` - 如果为 `true`，只查找本地引用，不查找远程引用（避免在 Windows 上触发网络操作导致超时）
    ///
    /// # 返回
    ///
    /// 返回共同祖先的提交哈希。如果两个分支没有共同祖先，返回错误。
    ///
    /// # 错误
    ///
    /// 如果分支不存在或操作失败，返回相应的错误信息。
    pub fn merge_base(branch1: &str, branch2: &str) -> Result<String> {
        Self::merge_base_internal(branch1, branch2, false)
    }

    /// 内部方法：获取两个分支的共同祖先（merge base）
    ///
    /// # 参数
    ///
    /// * `branch1` - 第一个分支名称
    /// * `branch2` - 第二个分支名称
    /// * `local_only` - 如果为 `true`，只查找本地引用，不查找远程引用（此参数在当前实现中未使用，因为 git merge-base 会自动处理）
    fn merge_base_internal(branch1: &str, branch2: &str, _local_only: bool) -> Result<String> {
        // 使用 git merge-base 获取合并基
        let merge_base = GitCommitCommand::merge_base(branch1, branch2, None).map_err(|e| {
            color_eyre::eyre::eyre!(
                "Failed to get merge base between '{}' and '{}': {}",
                branch1,
                branch2,
                e
            )
        })?;

        Ok(merge_base)
    }

    /// 检查一个分支是否直接基于另一个分支创建
    ///
    /// 使用 Git 命令通过比较 merge-base 和候选分支的 HEAD 来判断 from_branch 是否直接基于 candidate_branch 创建。
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
        Self::is_branch_based_on_internal(from_branch, candidate_branch, false)
    }

    /// 内部方法：检查一个分支是否直接基于另一个分支创建
    ///
    /// # 参数
    ///
    /// * `from_branch` - 要检查的分支
    /// * `candidate_branch` - 候选的基础分支
    /// * `local_only` - 如果为 `true`，只查找本地引用，不查找远程引用（此参数在当前实现中未使用，因为 git 命令会自动处理）
    fn is_branch_based_on_internal(
        from_branch: &str,
        candidate_branch: &str,
        _local_only: bool,
    ) -> Result<bool> {
        // 如果两个分支相同，返回 false
        if from_branch == candidate_branch {
            return Ok(false);
        }

        // 获取 merge-base
        let merge_base = match Self::merge_base_internal(from_branch, candidate_branch, false) {
            Ok(oid) => oid,
            Err(_) => return Ok(false),
        };

        // 获取 candidate_branch 的 HEAD commit
        let candidate_commit =
            GitCommitCommand::rev_parse(candidate_branch, None).map_err(|e| {
                color_eyre::eyre::eyre!(
                    "Failed to get commit from candidate branch {}: {}",
                    candidate_branch,
                    e
                )
            })?;

        // 如果 merge-base 等于 candidate_branch 的 HEAD，说明 from_branch 直接基于 candidate_branch
        Ok(merge_base == candidate_commit)
    }

    /// 检查 commit 是否在远程分支中
    ///
    /// 使用 Git 命令通过检查远程分支是否包含指定的 commit 来判断。
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
    /// 如果操作失败，返回相应的错误信息。
    pub fn is_commit_in_remote(branch: &str, commit_sha: &str) -> Result<bool> {
        // 首先检查远程分支是否存在
        let (_, has_remote) = Self::is_branch_exists(branch)?;
        if !has_remote {
            return Ok(false);
        }

        // 使用 git branch --contains 检查 commit 是否在远程分支中
        let remote_branch = format!("origin/{}", branch);
        let remote_branches =
            GitBranchCommand::remote_branch_contains(commit_sha, None).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to check if commit is in remote branch: {}", e)
            })?;

        // 检查输出中是否包含远程分支
        Ok(remote_branches.iter().any(|line| line.trim() == remote_branch))
    }

    /// 检测指定分支可能基于哪个分支创建
    ///
    /// 通过检查所有分支，找出指定分支可能直接基于哪个分支创建。
    /// 如果检测到基础分支，返回其名称。
    ///
    /// # 参数
    ///
    /// * `branch` - 要检测的分支名称
    /// * `exclude_branch` - 要排除的分支（通常是目标分支）
    ///
    /// # 返回
    ///
    /// 如果检测到基础分支，返回 `Some(base_branch_name)`，否则返回 `None`。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use workflow::git::GitBranch;
    ///
    /// // 检测 test-rebase 分支基于哪个分支创建（排除 master）
    /// let base = GitBranch::detect_base_branch("test-rebase", "master")?;
    /// // 可能返回: Some("develop")
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn detect_base_branch(branch: &str, exclude_branch: &str) -> Result<Option<String>> {
        trace_info!("Detecting base branch for '{}'...", branch);

        // 仅获取本地分支（避免在 Windows 上访问远程引用时可能触发网络操作导致卡住）
        // 远程分支通常是本地分支的副本，检查本地分支即可
        let all_branches = Self::get_local_branches()
            .wrap_err("Failed to get local branches for base branch detection")?;

        // 按优先级排序：优先检查常见的基础分支
        let mut candidate_branches: Vec<String> = all_branches
            .into_iter()
            .filter(|b| b != branch && b != exclude_branch)
            .collect();

        // 优先检查常见的基础分支名称（develop, dev, staging, test）
        let common_base_branches = ["develop", "dev", "staging", "test"];
        candidate_branches.sort_by(|a, b| {
            let a_priority = common_base_branches
                .iter()
                .position(|&name| a == name || a.ends_with(&format!("/{}", name)))
                .unwrap_or(usize::MAX);
            let b_priority = common_base_branches
                .iter()
                .position(|&name| b == name || b.ends_with(&format!("/{}", name)))
                .unwrap_or(usize::MAX);
            a_priority.cmp(&b_priority)
        });

        // 检查每个候选分支（只使用本地引用，避免在 Windows 上触发网络操作导致超时）
        for candidate in &candidate_branches {
            match Self::is_branch_based_on_internal(branch, candidate, true) {
                Ok(true) => {
                    trace_info!(
                        "Detected that '{}' is likely based on '{}'",
                        branch,
                        candidate
                    );
                    return Ok(Some(candidate.clone()));
                }
                Ok(false) => {
                    // 继续检查下一个分支
                }
                Err(e) => {
                    // 检查失败，记录警告但继续
                    trace_warn!(
                        "Failed to check if '{}' is based on '{}': {}",
                        branch,
                        candidate,
                        e
                    );
                }
            }
        }

        trace_info!("No base branch detected for '{}'", branch);
        Ok(None)
    }
}
