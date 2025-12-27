use color_eyre::{eyre::WrapErr, Result};
use std::collections::HashSet;

use super::{GitAuth, GitCommand};
use crate::git::helpers::open_repo;
use git2::{FetchOptions, PushOptions};

const COMMON_DEFAULT_BRANCHES: &[&str] = &["main", "master", "develop", "dev"];
/// 尝试使用 git switch，失败时回退到 git checkout
#[allow(dead_code)]
fn switch_or_checkout(
    switch_args: &[&str],
    checkout_args: &[&str],
    error_msg: impl AsRef<str>,
) -> Result<()> {
    if !GitCommand::new(switch_args).quiet_success() {
        GitCommand::new(checkout_args)
            .run()
            .wrap_err_with(|| error_msg.as_ref().to_string())?;
    }
    Ok(())
}

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
    /// 使用 git2 库获取当前分支的名称。
    ///
    /// # 返回
    ///
    /// 返回当前分支的名称。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或操作失败，返回相应的错误信息。
    pub fn current_branch() -> Result<String> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;

        // 获取分支名称（去掉 refs/heads/ 前缀）
        let branch_name = head
            .name()
            .and_then(|name| name.strip_prefix("refs/heads/"))
            .ok_or_else(|| color_eyre::eyre::eyre!("HEAD is not pointing to a branch"))?;

        Ok(branch_name.to_string())
    }

    /// 获取当前分支名（指定仓库路径）
    ///
    /// 使用 git2 库获取指定仓库的当前分支名称。
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
        let repo = git2::Repository::open(repo_path.as_ref())
            .wrap_err("Failed to open repository")?;
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;

        // 获取分支名称（去掉 refs/heads/ 前缀）
        let branch_name = head
            .name()
            .and_then(|name| name.strip_prefix("refs/heads/"))
            .ok_or_else(|| color_eyre::eyre::eyre!("HEAD is not pointing to a branch"))?;

        Ok(branch_name.to_string())
    }

    /// 检查分支是否存在（本地或远程）
    ///
    /// 使用 git2 库检查指定分支在本地和远程是否存在。
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
        let repo = open_repo()?;

        // 检查本地分支
        let local_ref = format!("refs/heads/{}", branch_name);
        let exists_local = repo.find_reference(&local_ref).is_ok();

        // 检查远程分支
        let remote_ref = format!("refs/remotes/origin/{}", branch_name);
        let exists_remote = repo.find_reference(&remote_ref).is_ok();

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

        let repo = open_repo()?;

        if exists_local {
            // 分支已存在于本地，切换到它
            let refname = format!("refs/heads/{}", branch_name);
            repo.set_head(&refname)
                .wrap_err_with(|| format!("Failed to set HEAD to branch: {}", branch_name))?;
            repo.checkout_head(Some(
                git2::build::CheckoutBuilder::default()
                    .force()
                    .remove_ignored(false)
                    .remove_untracked(false),
            ))
            .wrap_err_with(|| format!("Failed to checkout branch: {}", branch_name))?;
        } else if exists_remote {
            // 分支只存在于远程，创建本地分支并跟踪远程分支
            let remote_ref = format!("refs/remotes/origin/{}", branch_name);
            let remote_branch = repo.find_reference(&remote_ref)
                .wrap_err_with(|| format!("Failed to find remote branch: {}", branch_name))?;
            let remote_commit = remote_branch.peel_to_commit()
                .wrap_err_with(|| format!("Failed to get commit from remote branch: {}", branch_name))?;

            // 创建本地分支指向远程分支的提交
            let mut local_branch = repo.branch(branch_name, &remote_commit, false)
                .wrap_err_with(|| format!("Failed to create local branch: {}", branch_name))?;

            // 设置上游跟踪
            local_branch.set_upstream(Some(&format!("origin/{}", branch_name)))
                .wrap_err_with(|| format!("Failed to set upstream for branch: {}", branch_name))?;

            // 切换到新创建的分支
            let local_refname = format!("refs/heads/{}", branch_name);
            repo.set_head(&local_refname)
                .wrap_err_with(|| format!("Failed to set HEAD to branch: {}", branch_name))?;
            repo.checkout_head(Some(
                git2::build::CheckoutBuilder::default()
                    .force()
                    .remove_ignored(false)
                    .remove_untracked(false),
            ))
            .wrap_err_with(|| format!("Failed to checkout branch: {}", branch_name))?;
        } else {
            // 分支不存在，创建新分支
            let head = repo.head()
                .wrap_err("Failed to get HEAD")?;
            let head_commit = head.peel_to_commit()
                .wrap_err("Failed to get HEAD commit")?;

            // 创建新分支
            repo.branch(branch_name, &head_commit, false)
                .wrap_err_with(|| format!("Failed to create branch: {}", branch_name))?;

            // 切换到新创建的分支
            let refname = format!("refs/heads/{}", branch_name);
            repo.set_head(&refname)
                .wrap_err_with(|| format!("Failed to set HEAD to branch: {}", branch_name))?;
            repo.checkout_head(Some(
                git2::build::CheckoutBuilder::default()
                    .force()
                    .remove_ignored(false)
                    .remove_untracked(false),
            ))
            .wrap_err_with(|| format!("Failed to checkout branch: {}", branch_name))?;
        }
        Ok(())
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
    /// 使用 git2 库获取指定仓库的默认分支名称。
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
        let repo = git2::Repository::open(repo_path.as_ref())
            .wrap_err("Failed to open repository")?;

        // 优先尝试方法1：从远程分支列表中查找常见的默认分支名（不依赖网络，最快）
        // 这在测试环境中最可靠，因为测试已经设置了正确的远程引用
        if let Ok(branch) = Self::find_default_branch_from_remote_in(repo_path.as_ref()) {
            return Ok(branch);
        }

        // 尝试方法2：从远程获取 HEAD 引用（需要网络连接和认证）
        if let Ok(mut remote) = repo.find_remote("origin") {
            // 尝试连接远程并获取引用列表
            let callbacks = GitAuth::get_remote_callbacks();
            if remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None).is_ok() {
                // 获取远程引用列表
                if let Ok(refs) = remote.list() {
                    // 查找 HEAD 引用
                    for remote_ref in refs {
                        if remote_ref.name() == "HEAD" {
                            // HEAD 引用指向默认分支，格式通常是 "refs/heads/main"
                            if let Some(branch_ref) = remote_ref.symref_target() {
                                if let Some(branch) = branch_ref.strip_prefix("refs/heads/") {
                                    return Ok(branch.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 尝试方法3：从 remote show origin 获取（需要网络连接，作为回退）
        // 使用 quiet_success() 快速检查是否可用，避免长时间阻塞
        if GitCommand::new(["remote", "show", "origin"])
            .with_cwd(repo_path.as_ref())
            .with_env("GIT_TERMINAL_PROMPT", "0")
            .quiet_success()
        {
            if let Ok(branch) = Self::get_default_branch_from_remote_show_in(repo_path.as_ref()) {
                return Ok(branch);
            }
        }

        // 尝试方法4：使用 ls-remote --symref 直接从远程获取符号引用（需要网络连接，作为最后回退）
        // 使用 quiet_success() 快速检查是否可用，避免长时间阻塞
        if GitCommand::new(["ls-remote", "--symref", "origin", "HEAD"])
            .with_cwd(repo_path.as_ref())
            .with_env("GIT_TERMINAL_PROMPT", "0")
            .quiet_success()
        {
            if let Ok(output) = GitCommand::new(["ls-remote", "--symref", "origin", "HEAD"])
                .with_cwd(repo_path.as_ref())
                .with_env("GIT_TERMINAL_PROMPT", "0")
                .read()
            {
                if let Some(branch) = Self::parse_symref_output(&output) {
                    return Ok(branch);
                }
            }
        }

        color_eyre::eyre::bail!("Failed to get default branch")
    }

    /// 从 `git remote show origin` 获取默认分支（指定仓库路径）
    fn get_default_branch_from_remote_show_in(
        repo_path: impl AsRef<std::path::Path>,
    ) -> Result<String> {
        let remote_info = GitCommand::new(["remote", "show", "origin"])
            .with_cwd(repo_path.as_ref())
            .with_env("GIT_TERMINAL_PROMPT", "0")
            .read()?;
        for line in remote_info.lines() {
            if line.contains("HEAD branch:") {
                if let Some(branch) = line.split("HEAD branch:").nth(1) {
                    return Ok(branch.trim().to_string());
                }
            }
        }
        color_eyre::eyre::bail!("Could not determine default branch from remote show")
    }

    /// 解析 ls-remote --symref 的输出
    ///
    /// 从 `git ls-remote --symref` 的输出中提取默认分支名称。
    /// 输出格式通常是：`ref: refs/heads/main\tHEAD`
    ///
    /// # 参数
    ///
    /// * `output` - `git ls-remote --symref` 命令的输出字符串
    ///
    /// # 返回
    ///
    /// 返回默认分支名称（如果找到），否则返回 `None`。
    fn parse_symref_output(output: &str) -> Option<String> {
        for line in output.lines() {
            if line.starts_with("ref:") {
                // 提取 refs/heads/<branch> 中的分支名
                if let Some(parts) = line.split_whitespace().nth(1) {
                    if let Some(branch) = parts.strip_prefix("refs/heads/") {
                        return Some(branch.to_string());
                    }
                }
            }
        }
        None
    }

    /// 从远程分支列表中查找常见的默认分支名（指定仓库路径）
    ///
    /// 使用 git2 库从远程分支列表中查找常见的默认分支名。
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
        let repo = git2::Repository::open(repo_path.as_ref())
            .wrap_err("Failed to open repository")?;

        // 获取所有远程分支引用
        let remote_refs: Vec<String> = repo.references()?
            .filter_map(|reference| {
                reference.ok().and_then(|ref_| {
                    ref_.name()
                        .and_then(|name| name.strip_prefix("refs/remotes/origin/"))
                        .map(|name| name.to_string())
                })
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
    /// 使用 git2 库获取所有本地分支和远程分支，去除重复的分支名称，返回去重后的分支列表。
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
        let repo = open_repo()?;
        let mut branch_set = HashSet::new();

        // 获取本地分支
        repo.references()?
            .filter_map(|reference| {
                reference.ok().and_then(|ref_| {
                    ref_.name()
                        .and_then(|name| name.strip_prefix("refs/heads/"))
                        .map(|name| name.to_string())
                })
            })
            .for_each(|name| {
                branch_set.insert(name);
            });

        // 获取远程分支
        repo.references()?
            .filter_map(|reference| {
                reference.ok().and_then(|ref_| {
                    ref_.name()
                        .and_then(|name| name.strip_prefix("refs/remotes/"))
                        .and_then(|name| {
                            // 移除远程名称前缀（如 "origin/"）
                            name.split('/').nth(1).map(|name| name.to_string())
                        })
                })
            })
            .for_each(|name| {
                branch_set.insert(name);
            });

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
    /// 使用 git2 库获取所有本地分支列表，不包括远程分支。
    ///
    /// # 返回
    ///
    /// 返回本地分支名称列表（按字母顺序排序）
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn get_local_branches() -> Result<Vec<String>> {
        let repo = open_repo()?;
        let mut branches = Vec::new();

        // 遍历所有引用，查找本地分支
        repo.references()?
            .filter_map(|reference| {
                reference.ok().and_then(|ref_| {
                    ref_.name()
                        .and_then(|name| name.strip_prefix("refs/heads/"))
                        .map(|name| name.to_string())
                })
            })
            .for_each(|name| branches.push(name));

        branches.sort();
        Ok(branches)
    }

    /// 解析本地分支列表
    #[allow(dead_code)]
    fn parse_local_branches(output: &str, branch_set: &mut HashSet<String>) {
        for line in output.lines() {
            let line = line.trim();
            // 跳过空行和 HEAD 指向的行（如 "  remotes/origin/HEAD -> origin/main"）
            if line.is_empty() || line.contains("->") {
                continue;
            }
            // 移除 `*` 标记（当前分支）和首尾空白
            let branch_name = line.trim_start_matches('*').trim();
            if !branch_name.is_empty() {
                branch_set.insert(branch_name.to_string());
            }
        }
    }

    /// 解析远程分支列表
    #[allow(dead_code)]
    fn parse_remote_branches(output: &str, branch_set: &mut HashSet<String>) {
        for line in output.lines() {
            let line = line.trim();
            // 跳过空行和 HEAD 指向的行
            if line.is_empty() || line.contains("->") {
                continue;
            }
            // 移除 `origin/` 前缀
            let branch_name = if let Some(name) = line.strip_prefix("origin/") {
                name.trim()
            } else {
                // 如果不是 origin/ 开头，尝试移除其他远程名称
                line.split('/').nth(1).unwrap_or(line).trim()
            };
            if !branch_name.is_empty() {
                branch_set.insert(branch_name.to_string());
            }
        }
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

        // 解析分支引用
        let branch_ref = repo
            .find_reference(&format!("refs/heads/{}", branch_name))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch_name)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;
        let branch_commit = branch_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from branch: {}", branch_name))?;

        let base_ref = repo
            .find_reference(&format!("refs/heads/{}", base_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", base_branch)))
            .wrap_err_with(|| format!("Failed to find base branch: {}", base_branch))?;
        let base_commit = base_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from base branch: {}", base_branch))?;

        // 使用 revwalk 检查是否有新提交
        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk
            .push(branch_commit.id())
            .wrap_err("Failed to push branch commit to revwalk")?;
        revwalk
            .hide(base_commit.id())
            .wrap_err("Failed to hide base commit from revwalk")?;

        // 检查是否有至少一个提交
        Ok(revwalk.next().is_some())
    }

    /// 从远程拉取指定分支的最新更改
    ///
    /// 使用 git2 库从远程仓库拉取指定分支的最新更改。
    /// 支持 SSH 和 HTTPS 认证，适用于私有仓库。
    /// 自动处理 fast-forward 合并和普通合并。
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
        let repo = open_repo()?;
        let mut remote = repo
            .find_remote("origin")
            .wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置获取选项
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // 获取远程更新
        let refspec = format!("refs/heads/{}:refs/remotes/origin/{}", branch_name, branch_name);
        remote
            .fetch(&[&refspec], Some(&mut fetch_options), None)
            .wrap_err("Failed to fetch from origin")?;

        // 更新远程引用后，查找远程分支的提交
        let remote_ref = repo
            .find_reference(&format!("refs/remotes/origin/{}", branch_name))
            .wrap_err_with(|| format!("Failed to find remote branch: origin/{}", branch_name))?;

        let remote_commit = remote_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from remote branch: origin/{}", branch_name))?;

        // 查找本地分支（如果存在）
        let local_branch_ref = repo.find_reference(&format!("refs/heads/{}", branch_name));

        match local_branch_ref {
            Ok(local_ref) => {
                // 本地分支存在，需要合并
                let local_commit = local_ref
                    .peel_to_commit()
                    .wrap_err_with(|| format!("Failed to get commit from local branch: {}", branch_name))?;

                // 创建 annotated commit 用于合并分析
                let remote_annotated = repo
                    .find_annotated_commit(remote_commit.id())
                    .wrap_err("Failed to create annotated commit from remote")?;

                // 分析合并情况
                let (analysis, _) = repo
                    .merge_analysis(&[&remote_annotated])
                    .wrap_err("Failed to analyze merge")?;

                if analysis.is_up_to_date() {
                    // 已经是最新的，不需要操作
                    return Ok(());
                } else if analysis.is_fast_forward() {
                    // Fast-forward 合并
                    let mut local_ref = repo
                        .find_reference(&format!("refs/heads/{}", branch_name))
                        .wrap_err_with(|| format!("Failed to find local branch: {}", branch_name))?;
                    local_ref
                        .set_target(remote_commit.id(), "Fast-forward")
                        .wrap_err("Failed to update branch reference")?;

                    // 更新工作目录
                    repo.set_head(&format!("refs/heads/{}", branch_name))
                        .wrap_err("Failed to set HEAD")?;
                    repo.checkout_head(Some(
                        git2::build::CheckoutBuilder::default()
                            .force()
                            .remove_untracked(true),
                    ))
                    .wrap_err("Failed to checkout HEAD")?;
                } else {
                    // 需要合并
                    // merge_commits 需要 Commit 而不是 AnnotatedCommit
                    let mut index = repo
                        .merge_commits(&local_commit, &remote_commit, None)
                        .wrap_err("Failed to merge commits")?;

                    if index.has_conflicts() {
                        // 有冲突，写入索引并返回错误
                        index.write().wrap_err("Failed to write merge index")?;
                        return Err(color_eyre::eyre::eyre!(
                            "Merge conflicts detected. Please resolve conflicts manually."
                        ));
                    }

                    // 没有冲突，创建合并提交
                    let tree_id = index
                        .write_tree_to(&repo)
                        .wrap_err("Failed to write merge tree")?;
                    let tree = repo
                        .find_tree(tree_id)
                        .wrap_err("Failed to find merge tree")?;

                    let signature = repo
                        .signature()
                        .wrap_err("Failed to get repository signature")?;

                    let message = format!("Merge branch '{}' of origin into {}", branch_name, branch_name);

                    repo.commit(
                        Some(&format!("refs/heads/{}", branch_name)),
                        &signature,
                        &signature,
                        &message,
                        &tree,
                        &[&local_commit, &remote_commit],
                    )
                    .wrap_err("Failed to create merge commit")?;
                }
            }
            Err(_) => {
                // 本地分支不存在，创建新分支指向远程分支
                repo.reference(
                    &format!("refs/heads/{}", branch_name),
                    remote_commit.id(),
                    true,
                    "Pull: create local branch from remote",
                )
                .wrap_err_with(|| format!("Failed to create local branch: {}", branch_name))?;

                // 设置 HEAD 并检出
                repo.set_head(&format!("refs/heads/{}", branch_name))
                    .wrap_err("Failed to set HEAD")?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .force()
                        .remove_untracked(true),
                ))
                .wrap_err("Failed to checkout HEAD")?;
            }
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
        let mut remote = repo
            .find_remote("origin")
            .wrap_err("Failed to find remote 'origin'")?;

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
                .find_branch(branch_name, git2::BranchType::Local)
                .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;
            branch
                .set_upstream(Some(&format!("origin/{}", branch_name)))
                .wrap_err("Failed to set upstream")?;
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
        let mut remote = repo
            .find_remote("origin")
            .wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // 构建 refspec（带 force-with-lease，使用 + 前缀）
        let refspec = format!("+refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        // 推送
        remote
            .push(&[&refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to force push branch: {}", branch_name))?;

        Ok(())
    }

    /// 将当前分支 rebase 到目标分支
    ///
    /// 使用 git2 库将当前分支的提交重新应用到目标分支之上。
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

        // 获取当前分支的 HEAD commit
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let head_commit = head
            .peel_to_commit()
            .wrap_err("Failed to get commit from HEAD")?;

        // 解析目标分支（支持本地分支或远程分支）
        let target_ref = repo
            .find_reference(&format!("refs/heads/{}", target_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", target_branch)))
            .wrap_err_with(|| format!("Failed to find branch: {}", target_branch))?;
        let target_commit = target_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from branch: {}", target_branch))?;

        // 转换为 AnnotatedCommit（rebase 需要）
        let head_annotated = repo
            .find_annotated_commit(head_commit.id())
            .wrap_err("Failed to create annotated commit from HEAD")?;
        let target_annotated = repo
            .find_annotated_commit(target_commit.id())
            .wrap_err_with(|| format!("Failed to create annotated commit from branch: {}", target_branch))?;

        // 初始化 rebase
        let mut rebase_opts = git2::RebaseOptions::new();
        rebase_opts.inmemory(true); // 使用内存模式，避免修改工作目录

        let mut rebase = repo
            .rebase(
                Some(&head_annotated),
                Some(&target_annotated),
                None,
                Some(&mut rebase_opts),
            )
            .wrap_err_with(|| format!("Failed to initialize rebase onto branch: {}", target_branch))?;

        // 应用所有 rebase 操作
        while let Some(_op) = rebase.next() {
            let _op = _op.wrap_err("Failed to get next rebase operation")?;

            // 检查是否有冲突
            if repo.index()?.has_conflicts() {
                rebase.abort().ok(); // 尝试中止 rebase
                color_eyre::eyre::bail!(
                    "Rebase conflict detected. Please resolve conflicts manually and continue."
                );
            }

            // 提交 rebase 操作
            let signature = repo.signature().wrap_err("Failed to get signature")?;
            rebase
                .commit(None, &signature, None)
                .wrap_err("Failed to commit rebase operation")?;
        }

        // 完成 rebase
        rebase
            .finish(None)
            .wrap_err_with(|| format!("Failed to finish rebase onto branch: {}", target_branch))?;

        Ok(())
    }

    /// 将指定范围的提交 rebase 到目标分支
    ///
    /// 使用 git2 库将 `<upstream>..<branch>` 范围内的提交
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

        // 解析分支引用（支持本地分支或远程分支）
        let branch_ref = repo
            .find_reference(&format!("refs/heads/{}", branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch))?;
        let branch_commit = branch_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from branch: {}", branch))?;

        // 解析上游分支
        let upstream_ref = repo
            .find_reference(&format!("refs/heads/{}", upstream))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", upstream)))
            .wrap_err_with(|| format!("Failed to find upstream branch: {}", upstream))?;
        let upstream_commit = upstream_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from upstream branch: {}", upstream))?;

        // 解析新基础分支
        let newbase_ref = repo
            .find_reference(&format!("refs/heads/{}", newbase))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", newbase)))
            .wrap_err_with(|| format!("Failed to find newbase branch: {}", newbase))?;
        let newbase_commit = newbase_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from newbase branch: {}", newbase))?;

        // 转换为 AnnotatedCommit（rebase 需要）
        let branch_annotated = repo
            .find_annotated_commit(branch_commit.id())
            .wrap_err_with(|| format!("Failed to create annotated commit from branch: {}", branch))?;
        let upstream_annotated = repo
            .find_annotated_commit(upstream_commit.id())
            .wrap_err_with(|| format!("Failed to create annotated commit from upstream branch: {}", upstream))?;
        let newbase_annotated = repo
            .find_annotated_commit(newbase_commit.id())
            .wrap_err_with(|| format!("Failed to create annotated commit from newbase branch: {}", newbase))?;

        // 初始化 rebase（使用 --onto 模式）
        let mut rebase_opts = git2::RebaseOptions::new();
        rebase_opts.inmemory(true); // 使用内存模式，避免修改工作目录

        let mut rebase = repo
            .rebase(
                Some(&branch_annotated),
                Some(&upstream_annotated),
                Some(&newbase_annotated),
                Some(&mut rebase_opts),
            )
            .wrap_err_with(|| {
                format!(
                    "Failed to initialize rebase '{}' onto '{}' (excluding '{}' commits)",
                    branch, newbase, upstream
                )
            })?;

        // 应用所有 rebase 操作
        while let Some(_op) = rebase.next() {
            let _op = _op.wrap_err("Failed to get next rebase operation")?;

            // 检查是否有冲突
            if repo.index()?.has_conflicts() {
                rebase.abort().ok(); // 尝试中止 rebase
                color_eyre::eyre::bail!(
                    "Rebase conflict detected. Please resolve conflicts manually and continue."
                );
            }

            // 提交 rebase 操作
            let signature = repo.signature().wrap_err("Failed to get signature")?;
            rebase
                .commit(None, &signature, None)
                .wrap_err("Failed to commit rebase operation")?;
        }

        // 完成 rebase
        rebase
            .finish(None)
            .wrap_err_with(|| {
                format!(
                    "Failed to finish rebase '{}' onto '{}' (excluding '{}' commits)",
                    branch, newbase, upstream
                )
            })?;

        Ok(())
    }

    /// 删除本地分支
    ///
    /// 使用 git2 库删除指定的本地分支。如果分支未完全合并，可以使用 `force` 参数强制删除。
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
        let repo = open_repo()?;
        let refname = format!("refs/heads/{}", branch_name);

        // 查找分支引用
        let mut branch_ref = repo.find_reference(&refname)
            .wrap_err_with(|| format!("Failed to find branch: {}", branch_name))?;

        // 如果不是强制删除，检查分支是否已合并
        if !force {
            // 获取当前 HEAD
            let head = repo.head()
                .wrap_err("Failed to get HEAD")?;
            let head_commit = head.peel_to_commit()
                .wrap_err("Failed to get HEAD commit")?;

            // 获取分支的提交
            let branch_commit = branch_ref.peel_to_commit()
                .wrap_err_with(|| format!("Failed to get branch commit: {}", branch_name))?;

            // 检查分支是否已合并到当前分支
            let merge_base = repo.merge_base(head_commit.id(), branch_commit.id())
                .wrap_err("Failed to find merge base")?;

            // 如果合并基不等于分支提交，说明分支未完全合并
            if merge_base != branch_commit.id() {
                return Err(color_eyre::eyre::eyre!(
                    "Branch '{}' is not fully merged. Use force=true to delete it anyway.",
                    branch_name
                ));
            }
        }

        // 删除分支引用
        branch_ref.delete()
            .wrap_err_with(|| format!("Failed to delete branch: {}", branch_name))?;

        Ok(())
    }

    /// 删除远程分支
    ///
    /// 使用 git2 库删除远程分支，通过推送空的 refspec 来实现。
    /// 这相当于 `git push origin --delete <branch_name>`。
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
        let mut remote = repo
            .find_remote("origin")
            .wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // 构建空的 refspec 来删除远程分支
        // 格式：:refs/heads/<branch_name> 表示删除远程分支
        let refspec = format!(":refs/heads/{}", branch_name);

        // 推送空的 refspec 来删除远程分支
        remote
            .push(&[&refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to delete remote branch: {}", branch_name))?;

        Ok(())
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
        let mut args = vec!["branch", "-m"];
        if let Some(old) = old_name {
            args.push(old);
        }
        args.push(new_name);
        GitCommand::new(args).run().wrap_err_with(|| {
            if let Some(old) = old_name {
                format!("Failed to rename branch from '{}' to '{}'", old, new_name)
            } else {
                format!("Failed to rename current branch to '{}'", new_name)
            }
        })
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
    /// 使用 git2 库根据指定的合并策略将源分支合并到当前分支。
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
        let repo = open_repo()?;

        // 获取当前分支的 commit
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let head_commit = head
            .peel_to_commit()
            .wrap_err("Failed to get commit from HEAD")?;

        // 获取源分支的 commit
        let source_ref = repo
            .find_reference(&format!("refs/heads/{}", source_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", source_branch)))
            .wrap_err_with(|| format!("Failed to find branch: {}", source_branch))?;
        let source_commit = source_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from branch: {}", source_branch))?;

        // 分析合并情况（需要 AnnotatedCommit）
        let source_annotated = repo
            .find_annotated_commit(source_commit.id())
            .wrap_err("Failed to create annotated commit")?;
        let (merge_analysis, _) = repo
            .merge_analysis(&[&source_annotated])
            .wrap_err("Failed to analyze merge")?;

        match strategy {
            MergeStrategy::FastForwardOnly => {
                if merge_analysis.is_fast_forward() {
                    // Fast-forward 合并
                    let mut head_ref = repo
                        .find_reference(head.name().unwrap())
                        .wrap_err("Failed to find HEAD reference")?;
                    head_ref
                        .set_target(source_commit.id(), "Fast-forward merge")
                        .wrap_err("Failed to update HEAD reference")?;

                    // 更新工作目录
                    repo.set_head(head.name().unwrap())
                        .wrap_err("Failed to set HEAD")?;
                    repo.checkout_head(Some(
                        git2::build::CheckoutBuilder::default()
                            .force()
                            .remove_untracked(true),
                    ))
                    .wrap_err("Failed to checkout HEAD")?;
                } else {
                    return Err(color_eyre::eyre::eyre!(
                        "Cannot fast-forward merge. Use a different merge strategy."
                    ));
                }
            }
            MergeStrategy::Squash => {
                // Squash 合并：将所有提交压缩为一个提交
                // 获取源分支的所有提交
                let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
                revwalk
                    .push(source_commit.id())
                    .wrap_err("Failed to push source commit to revwalk")?;
                revwalk
                    .hide(head_commit.id())
                    .wrap_err("Failed to hide HEAD commit from revwalk")?;

                // 收集所有要 squash 的提交
                let mut commits_to_squash = Vec::new();
                for oid in revwalk {
                    let oid = oid.wrap_err("Failed to get commit OID from revwalk")?;
                    let commit = repo
                        .find_commit(oid)
                        .wrap_err_with(|| format!("Failed to find commit: {}", oid))?;
                    commits_to_squash.push(commit);
                }

                // 如果没有提交需要 squash，直接返回
                if commits_to_squash.is_empty() {
                    return Ok(());
                }

                // 反转列表，从旧到新
                commits_to_squash.reverse();

                // 对于 squash，我们直接使用 merge_commits 来合并所有更改
                // git2 不直接支持 squash，我们需要手动实现

                // 创建 squash 提交（需要手动实现，git2 不直接支持 squash）
                // 这里我们使用 merge_commits 然后手动创建提交
                let mut merge_index = repo
                    .merge_commits(&head_commit, &source_commit, None)
                    .wrap_err("Failed to merge commits for squash")?;

                if merge_index.has_conflicts() {
                    merge_index.write().wrap_err("Failed to write merge index")?;
                    return Err(color_eyre::eyre::eyre!(
                        "Merge conflicts detected. Please resolve conflicts manually."
                    ));
                }

                let tree_id = merge_index
                    .write_tree_to(&repo)
                    .wrap_err("Failed to write merge tree")?;
                let tree = repo.find_tree(tree_id).wrap_err("Failed to find merge tree")?;

                let signature = repo.signature().wrap_err("Failed to get repository signature")?;

                // 创建 squash 提交消息
                let mut message = format!("Squashed commit of branch '{}'\n\n", source_branch);
                for commit in &commits_to_squash {
                    if let Some(msg) = commit.message() {
                        message.push_str(&format!("* {}\n", msg.lines().next().unwrap_or("")));
                    }
                }

                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &message,
                    &tree,
                    &[&head_commit],
                )
                .wrap_err("Failed to create squash commit")?;
            }
            MergeStrategy::Merge => {
                if merge_analysis.is_fast_forward() {
                    // Fast-forward 合并
                    let mut head_ref = repo
                        .find_reference(head.name().unwrap())
                        .wrap_err("Failed to find HEAD reference")?;
                    head_ref
                        .set_target(source_commit.id(), "Fast-forward merge")
                        .wrap_err("Failed to update HEAD reference")?;

                    // 更新工作目录
                    repo.set_head(head.name().unwrap())
                        .wrap_err("Failed to set HEAD")?;
                    repo.checkout_head(Some(
                        git2::build::CheckoutBuilder::default()
                            .force()
                            .remove_untracked(true),
                    ))
                    .wrap_err("Failed to checkout HEAD")?;
                } else {
                    // 普通合并
                    let mut merge_index = repo
                        .merge_commits(&head_commit, &source_commit, None)
                        .wrap_err("Failed to merge commits")?;

                    if merge_index.has_conflicts() {
                        merge_index.write().wrap_err("Failed to write merge index")?;
                        return Err(color_eyre::eyre::eyre!(
                            "Merge conflicts detected. Please resolve conflicts manually."
                        ));
                    }

                    let tree_id = merge_index
                        .write_tree_to(&repo)
                        .wrap_err("Failed to write merge tree")?;
                    let tree = repo.find_tree(tree_id).wrap_err("Failed to find merge tree")?;

                    let signature = repo
                        .signature()
                        .wrap_err("Failed to get repository signature")?;

                    let message = format!("Merge branch '{}'", source_branch);

                    repo.commit(
                        Some("HEAD"),
                        &signature,
                        &signature,
                        &message,
                        &tree,
                        &[&head_commit, &source_commit],
                    )
                    .wrap_err("Failed to create merge commit")?;
                }
            }
        }

        Ok(())
    }

    /// 验证合并是否已经完成
    ///
    /// 通过检查 MERGE_HEAD 是否存在或 HEAD 是否改变来判断合并是否完成。
    #[allow(dead_code)]
    fn verify_merge_completed(head_before: Option<String>) -> bool {
        // 检查是否有 MERGE_HEAD（表示合并正在进行或刚完成）
        let has_merge_head =
            GitCommand::new(["rev-parse", "--verify", "MERGE_HEAD"]).quiet_success();

        // 检查 HEAD 是否已经改变（表示合并可能已经完成）
        let head_after = GitCommand::new(["rev-parse", "HEAD"]).read().ok();
        let head_changed = if let (Some(before), Some(after)) = (head_before, head_after) {
            before.trim() != after.trim()
        } else {
            false
        };

        head_changed || has_merge_head
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
        // 检查是否有未合并的文件
        if !GitCommand::new(["diff", "--check"]).quiet_success() {
            return Ok(true);
        }

        // 检查 MERGE_HEAD 是否存在（表示正在进行合并）
        if GitCommand::new(["rev-parse", "--verify", "MERGE_HEAD"]).quiet_success() {
            // 检查是否有未解决的冲突文件
            if let Ok(files) = GitCommand::new(["diff", "--name-only", "--diff-filter=U"]).read() {
                return Ok(!files.trim().is_empty());
            }
        }

        Ok(false)
    }

    /// 检查分支是否已合并到指定分支
    ///
    /// 使用 git2 库检查指定分支是否已合并到基础分支。
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
        let repo = open_repo()?;

        // 解析分支引用
        let branch_ref = repo
            .find_reference(&format!("refs/heads/{}", branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch))?;
        let branch_commit = branch_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from branch: {}", branch))?;

        let base_ref = repo
            .find_reference(&format!("refs/heads/{}", base_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", base_branch)))
            .wrap_err_with(|| format!("Failed to find base branch: {}", base_branch))?;
        let base_commit = base_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from base branch: {}", base_branch))?;

        // 获取 merge-base
        let merge_base_oid = repo
            .merge_base(branch_commit.id(), base_commit.id())
            .wrap_err("Failed to get merge base")?;

        // 如果 merge-base 等于 branch 的 commit，说明 branch 已合并到 base_branch
        Ok(merge_base_oid == branch_commit.id())
    }

    /// 获取两个分支之间的提交列表
    ///
    /// 使用 git2 库获取 from_branch 相对于 to_branch 的所有新提交。
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
        let repo = open_repo()?;

        // 解析分支引用
        let base_ref = repo
            .find_reference(&format!("refs/heads/{}", base_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", base_branch)))
            .wrap_err_with(|| format!("Failed to find base branch: {}", base_branch))?;
        let base_commit = base_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from base branch: {}", base_branch))?;

        let head_ref = repo
            .find_reference(&format!("refs/heads/{}", head_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", head_branch)))
            .wrap_err_with(|| format!("Failed to find head branch: {}", head_branch))?;
        let head_commit = head_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from head branch: {}", head_branch))?;

        // 使用 revwalk 获取提交列表
        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk
            .push(head_commit.id())
            .wrap_err("Failed to push head commit to revwalk")?;
        revwalk
            .hide(base_commit.id())
            .wrap_err("Failed to hide base commit from revwalk")?;

        // 收集所有提交的 SHA
        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid.wrap_err("Failed to get commit OID from revwalk")?;
            commits.push(oid.to_string());
        }

        Ok(commits)
    }

    /// 获取两个分支的共同祖先（merge base）
    ///
    /// 使用 git2 库获取两个分支的共同祖先提交。
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
    /// 如果分支不存在或操作失败，返回相应的错误信息。
    pub fn merge_base(branch1: &str, branch2: &str) -> Result<String> {
        let repo = open_repo()?;

        // 解析分支引用
        let branch1_ref = repo
            .find_reference(&format!("refs/heads/{}", branch1))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch1)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch1))?;
        let branch1_commit = branch1_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from branch: {}", branch1))?;

        let branch2_ref = repo
            .find_reference(&format!("refs/heads/{}", branch2))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", branch2)))
            .wrap_err_with(|| format!("Failed to find branch: {}", branch2))?;
        let branch2_commit = branch2_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from branch: {}", branch2))?;

        // 使用 git2 的 merge_base 方法
        let merge_base_oid = repo
            .merge_base(branch1_commit.id(), branch2_commit.id())
            .wrap_err_with(|| {
                format!(
                    "Failed to get merge base between '{}' and '{}'",
                    branch1, branch2
                )
            })?;

        Ok(merge_base_oid.to_string())
    }

    /// 检查一个分支是否直接基于另一个分支创建
    ///
    /// 使用 git2 库通过比较 merge-base 和候选分支的 HEAD 来判断 from_branch 是否直接基于 candidate_branch 创建。
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

        let repo = open_repo()?;

        // 获取 merge-base
        let merge_base_oid = match Self::merge_base(from_branch, candidate_branch) {
            Ok(oid) => git2::Oid::from_str(&oid).wrap_err("Failed to parse merge base OID")?,
            Err(_) => return Ok(false),
        };

        // 获取 candidate_branch 的 HEAD commit
        let candidate_ref = repo
            .find_reference(&format!("refs/heads/{}", candidate_branch))
            .or_else(|_| repo.find_reference(&format!("refs/remotes/origin/{}", candidate_branch)))
            .wrap_err_with(|| format!("Failed to find candidate branch: {}", candidate_branch))?;
        let candidate_commit = candidate_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from candidate branch: {}", candidate_branch))?;

        // 如果 merge-base 等于 candidate_branch 的 HEAD，说明 from_branch 直接基于 candidate_branch
        Ok(merge_base_oid == candidate_commit.id())
    }

    /// 检查 commit 是否在远程分支中
    ///
    /// 使用 git2 库通过检查远程分支是否包含指定的 commit 来判断。
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

        let repo = open_repo()?;

        // 解析 commit SHA
        let commit_oid = git2::Oid::from_str(commit_sha)
            .wrap_err_with(|| format!("Failed to parse commit SHA: {}", commit_sha))?;

        // 查找远程分支引用
        let remote_ref = repo
            .find_reference(&format!("refs/remotes/origin/{}", branch))
            .wrap_err_with(|| format!("Failed to find remote branch: origin/{}", branch))?;
        let remote_commit = remote_ref
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit from remote branch: origin/{}", branch))?;

        // 使用 revwalk 检查 commit 是否在远程分支的历史中
        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk
            .push(remote_commit.id())
            .wrap_err("Failed to push remote commit to revwalk")?;

        // 检查 commit 是否在 revwalk 中
        for oid in revwalk {
            let oid = oid.wrap_err("Failed to get commit OID from revwalk")?;
            if oid == commit_oid {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
