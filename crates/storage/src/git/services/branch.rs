//! 分支业务逻辑服务
//!
//! 提供分支相关的业务逻辑实现。

use std::collections::HashMap;

use git2::{BranchType, ErrorCode, PushOptions, Repository};

use super::GitContext;
use domain::{BranchInfo, GitError};

/// 分支服务接口
pub trait BranchService: Send + Sync {
    /// 创建分支
    fn create_branch(&self, name: &str) -> Result<(), GitError>;

    /// 删除本地分支
    fn delete_local_branch(&self, name: &str, force: bool) -> Result<(), GitError>;

    /// 删除远程分支
    fn delete_remote_branch(&self, name: &str) -> Result<(), GitError>;

    /// 重命名分支
    fn rename_branch(&self, old_name: Option<&str>, new_name: &str) -> Result<(), GitError>;

    /// 获取分支列表
    fn list_branches(&self, remove_prefix: bool, all: bool) -> Result<Vec<BranchInfo>, GitError>;

    /// 切换分支
    fn checkout_branch(&self, name: &str) -> Result<(), GitError>;

    /// 获取当前分支名
    fn get_current_branch(&self) -> Result<String, GitError>;

    /// 检查分支是否存在 (本地, 远程)
    fn has_branch(&self, name: &str) -> Result<(bool, bool), GitError>;

    /// 获取默认分支
    fn get_default_branch(&self) -> Result<String, GitError>;

    /// 推断当前分支的目标合并分支
    ///
    /// 使用组合策略推断当前分支应该合并到哪个分支：
    /// 1. 优先从 reflog 查找分支创建来源（最准确但可能不存在）
    /// 2. 使用 merge base 分析找到最近的候选分支
    /// 3. 如果都失败，返回 None
    ///
    /// # 参数
    /// - `current_branch`: 当前分支名称
    ///
    /// # 返回
    /// - `Ok(Some(branch_name))`: 推断出的目标分支
    /// - `Ok(None)`: 无法推断
    /// - `Err`: 操作失败
    fn infer_target_branch(&self, current_branch: &str) -> Result<Option<String>, GitError>;
}

/// 分支服务实现
pub struct BranchServiceImpl {
    ctx: GitContext,
}

impl BranchServiceImpl {
    /// 创建新的分支服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
    }
}

impl BranchService for BranchServiceImpl {
    fn create_branch(&self, name: &str) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        // 获取 HEAD 指向的提交
        let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let commit = head.peel_to_commit().map_err(|_| {
            GitError::OperationFailed("Cannot get HEAD commit, repository may be empty".into())
        })?;

        // 创建分支
        repo.branch(name, &commit, false).map_err(|e| {
            if e.code() == ErrorCode::Exists {
                GitError::OperationFailed(format!("Branch '{}' already exists", name))
            } else {
                GitError::OperationFailed(e.to_string())
            }
        })?;

        Ok(())
    }

    fn delete_local_branch(&self, name: &str, force: bool) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        let mut branch = repo
            .find_branch(name, BranchType::Local)
            .map_err(|_| GitError::BranchNotFound(name.to_string()))?;

        // 检查是否为当前分支
        if branch.is_head() {
            return Err(GitError::OperationFailed(format!(
                "Cannot delete the current branch: {}",
                name
            )));
        }

        // 检查是否已合并（如果不强制删除）
        if !force {
            let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;
            let head_commit =
                head.peel_to_commit().map_err(|e| GitError::OperationFailed(e.to_string()))?;
            let branch_commit = branch
                .get()
                .peel_to_commit()
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;

            let is_merged =
                repo.graph_descendant_of(head_commit.id(), branch_commit.id()).unwrap_or(false);

            if !is_merged {
                return Err(GitError::BranchNotFullyMerged(name.to_string()));
            }
        }

        branch.delete().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    fn delete_remote_branch(&self, name: &str) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        let mut remote =
            repo.find_remote("origin").map_err(|e| GitError::RemoteError(e.to_string()))?;

        // 使用空引用删除远程分支
        let refspec = format!(":refs/heads/{}", name);

        let callbacks = GitContext::create_callbacks();
        let mut opts = PushOptions::new();
        opts.remote_callbacks(callbacks);

        remote.push(&[&refspec], Some(&mut opts)).map_err(|e| {
            // 检查是否是因为远程分支不存在
            if e.code() == ErrorCode::NotFound || e.code() == ErrorCode::Locked {
                GitError::BranchNotFound(format!("origin/{}", name))
            } else {
                GitError::RemoteError(e.to_string())
            }
        })?;

        Ok(())
    }

    fn rename_branch(&self, old_name: Option<&str>, new_name: &str) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        let mut branch = if let Some(name) = old_name {
            repo.find_branch(name, BranchType::Local)
                .map_err(|_| GitError::BranchNotFound(name.to_string()))?
        } else {
            // 重命名当前分支
            let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;
            if !head.is_branch() {
                return Err(GitError::OperationFailed(
                    "HEAD is in detached state".into(),
                ));
            }
            let branch_name = head
                .shorthand()
                .ok_or_else(|| GitError::OperationFailed("Invalid branch name".into()))?;
            repo.find_branch(branch_name, BranchType::Local)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?
        };

        branch.rename(new_name, false).map_err(|e| {
            if e.code() == ErrorCode::Exists {
                GitError::OperationFailed(format!("Branch '{}' already exists", new_name))
            } else {
                GitError::OperationFailed(e.to_string())
            }
        })?;

        Ok(())
    }

    fn list_branches(&self, remove_prefix: bool, all: bool) -> Result<Vec<BranchInfo>, GitError> {
        let repo = self.ctx.repository();

        let mut branches: HashMap<String, (bool, bool)> = HashMap::new(); // (has_local, has_remote)

        // 收集本地分支
        let local_branches = repo
            .branches(Some(BranchType::Local))
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        for branch_result in local_branches {
            let (branch, _) =
                branch_result.map_err(|e| GitError::OperationFailed(e.to_string()))?;
            if let Ok(Some(name)) = branch.name() {
                let formatted_name = self.format_branch_name(name, remove_prefix);
                branches.entry(formatted_name).or_insert((false, false)).0 = true;
            }
        }

        // 如果需要包含远程分支
        if all {
            let remote_branches = repo
                .branches(Some(BranchType::Remote))
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;

            for branch_result in remote_branches {
                let (branch, _) =
                    branch_result.map_err(|e| GitError::OperationFailed(e.to_string()))?;
                if let Ok(Some(name)) = branch.name() {
                    // 过滤掉 origin/HEAD（符号引用，不是真正的分支）
                    if name == "origin/HEAD" {
                        continue;
                    }

                    // 移除 origin/ 前缀获取分支短名
                    let short_name = name.strip_prefix("origin/").unwrap_or(name);
                    let formatted_name = self.format_branch_name(short_name, remove_prefix);

                    branches.entry(formatted_name).or_insert((false, false)).1 = true;
                }
            }
        }

        // 构建结果列表
        let mut result: Vec<BranchInfo> = branches
            .into_iter()
            .map(|(name, (has_local, has_remote))| {
                let display_name = if has_remote {
                    // 有远程分支，用 * 标记
                    format!("* {}", name)
                } else {
                    // 只有本地分支，用两个空格
                    format!("  {}", name)
                };

                // 根据本地/远程状态创建分支信息
                // name 字段保存不带 origin/ 前缀的短名称
                // 这样 has_branch 可以正确检查本地和远程
                if has_local {
                    let mut info = BranchInfo::local(name);
                    info.display_name = display_name;
                    info.is_remote = has_remote;
                    info
                } else {
                    // 仅存在于远程的分支
                    BranchInfo::remote(name, display_name)
                }
            })
            .collect();

        // 按显示名称排序
        result.sort_by(|a, b| a.display_name.cmp(&b.display_name));

        Ok(result)
    }

    fn checkout_branch(&self, name: &str) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        // 检查是否已是当前分支
        if let Ok(head) = repo.head() {
            if head.is_branch() {
                if let Some(current) = head.shorthand() {
                    if current == name {
                        return Ok(());
                    }
                }
            }
        }

        // 检查分支是否存在（本地或远程）
        let local_exists = repo.find_branch(name, BranchType::Local).is_ok();
        let remote_name = format!("origin/{}", name);
        let remote_exists = repo.find_branch(&remote_name, BranchType::Remote).is_ok();

        if local_exists {
            // 切换到本地分支
            let branch = repo
                .find_branch(name, BranchType::Local)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            let reference = branch.get();
            let refname = reference
                .name()
                .ok_or_else(|| GitError::OperationFailed("Invalid reference name".into()))?;

            self.checkout_to_ref(&repo, refname)?;
        } else if remote_exists {
            // 从远程分支创建本地分支
            let remote_branch = repo
                .find_branch(&remote_name, BranchType::Remote)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            let commit = remote_branch
                .get()
                .peel_to_commit()
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;

            // 创建本地分支
            repo.branch(name, &commit, false)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;

            // 切换到新分支
            let refname = format!("refs/heads/{}", name);
            self.checkout_to_ref(&repo, &refname)?;
        } else {
            // 创建新分支
            self.create_branch(name)?;

            // 切换到新分支
            let refname = format!("refs/heads/{}", name);
            self.checkout_to_ref(&repo, &refname)?;
        }

        Ok(())
    }

    fn get_current_branch(&self) -> Result<String, GitError> {
        let repo = self.ctx.repository();

        let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        if !head.is_branch() {
            return Err(GitError::OperationFailed(
                "HEAD is in detached state".into(),
            ));
        }

        head.shorthand()
            .map(String::from)
            .ok_or_else(|| GitError::OperationFailed("Cannot get branch name".into()))
    }

    fn has_branch(&self, name: &str) -> Result<(bool, bool), GitError> {
        let repo = self.ctx.repository();

        let local = repo.find_branch(name, BranchType::Local).is_ok();
        let remote_name = format!("origin/{}", name);
        let remote = repo.find_branch(&remote_name, BranchType::Remote).is_ok();

        Ok((local, remote))
    }

    fn get_default_branch(&self) -> Result<String, GitError> {
        let repo = self.ctx.repository();

        // 尝试从 refs/remotes/origin/HEAD 获取
        if let Ok(reference) = repo.find_reference("refs/remotes/origin/HEAD") {
            if let Some(target) = reference.symbolic_target() {
                if let Some(branch_name) = target.strip_prefix("refs/remotes/origin/") {
                    return Ok(branch_name.to_string());
                }
            }
        }

        // 尝试查询远程
        if let Ok(remote) = repo.find_remote("origin") {
            if let Ok(buf) = remote.default_branch() {
                if let Some(branch_name) = buf.as_str() {
                    let short_name = branch_name.strip_prefix("refs/heads/").unwrap_or(branch_name);
                    return Ok(short_name.to_string());
                }
            }
        }

        // 回退：检查常见的默认分支名
        const COMMON_DEFAULTS: &[&str] = &["main", "master", "develop", "dev"];
        for default_name in COMMON_DEFAULTS {
            let remote_ref = format!("origin/{}", default_name);
            if repo.find_branch(&remote_ref, BranchType::Remote).is_ok() {
                return Ok(default_name.to_string());
            }
        }

        Err(GitError::OperationFailed(
            "Cannot determine default branch".into(),
        ))
    }

    fn infer_target_branch(&self, current_branch: &str) -> Result<Option<String>, GitError> {
        // 策略1: 尝试从 reflog 推断（最准确）
        if let Some(target) = self.infer_from_reflog(current_branch)? {
            return Ok(Some(target));
        }

        // 策略2: 使用 merge base 分析（可靠）
        if let Some(target) = self.infer_from_merge_base(current_branch)? {
            return Ok(Some(target));
        }

        // 无法推断
        Ok(None)
    }
}

impl BranchServiceImpl {
    /// 切换到指定的引用
    ///
    /// 将工作区和 HEAD 切换到指定的引用（分支或提交）。
    ///
    /// # 参数
    /// - `repo`: Git 仓库引用（需要外部传入以避免死锁）
    /// - `refname`: 引用名称（如 `refs/heads/main`）
    fn checkout_to_ref(&self, repo: &Repository, refname: &str) -> Result<(), GitError> {
        let obj = repo
            .revparse_single(refname)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        repo.checkout_tree(&obj, None)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        repo.set_head(refname).map_err(|e| GitError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    /// 格式化分支名称
    ///
    /// # 参数
    /// - `name`: 原始分支名称
    /// - `remove_prefix`: 是否移除前缀
    ///
    /// # 返回
    /// 格式化后的分支名称
    fn format_branch_name(&self, name: &str, remove_prefix: bool) -> String {
        if remove_prefix {
            // 移除前缀（如 feature/xxx -> xxx 或 origin/feature/xxx -> xxx）
            name.strip_prefix("origin/")
                .unwrap_or(name)
                .rsplit('/')
                .next()
                .unwrap_or(name)
                .rsplit("--")
                .next()
                .unwrap_or(name)
                .to_string()
        } else {
            name.to_string()
        }
    }

    /// 从 reflog 推断目标分支
    ///
    /// 读取分支的 reflog，从最早的记录中提取分支创建来源。
    fn infer_from_reflog(&self, branch_name: &str) -> Result<Option<String>, GitError> {
        let repo = self.ctx.repository();
        let ref_name = format!("refs/heads/{}", branch_name);

        // 尝试读取 reflog
        let reflog = match repo.reflog(&ref_name) {
            Ok(log) => log,
            Err(_) => return Ok(None), // reflog 不存在
        };

        // 从最早的记录开始查找（reflog.iter() 从新到旧，使用 rev() 反转）
        for entry in reflog.iter().rev().take(5) {
            // 只查看前5条记录
            if let Some(message) = entry.message() {
                if let Some(source) = Self::extract_source_branch(message) {
                    // 验证源分支是否存在（用当前已持有的 repo，避免在持有锁时再调 branch_exists 导致死锁）
                    if repo.find_branch(&source, BranchType::Local).is_ok() {
                        return Ok(Some(source));
                    }
                }
            }
        }

        Ok(None)
    }

    /// 从 merge base 推断目标分支
    ///
    /// 计算当前分支与候选分支的 merge base，选择最"近"的分支。
    ///
    /// 策略：
    /// 1. 首先检查所有本地分支，找出 merge base 等于候选分支 HEAD 的情况
    ///    （这意味着当前分支是从该分支直接创建的，是最准确的推断）
    /// 2. 如果找到多个精确匹配，选择 commit 时间最新的（最近的父分支）
    /// 3. 如果没有精确匹配，则选择 merge base 时间最新的主线分支
    fn infer_from_merge_base(&self, current_branch: &str) -> Result<Option<String>, GitError> {
        let repo = self.ctx.repository();

        // 获取当前分支的 commit
        let current_ref = repo
            .find_branch(current_branch, BranchType::Local)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let current_commit = current_ref
            .get()
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 主线分支优先级列表
        const MAIN_BRANCHES: &[&str] = &["develop", "master", "main"];

        // 收集所有本地分支（排除当前分支）
        let all_branches = repo
            .branches(Some(BranchType::Local))
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 精确匹配：merge base == 候选分支 HEAD，记录 (分支名, commit时间)
        let mut exact_matches: Vec<(String, i64)> = Vec::with_capacity(4);
        // 备选：记录主线分支的 merge base 时间
        let mut main_branch_candidates: Vec<(String, i64)> =
            Vec::with_capacity(MAIN_BRANCHES.len());

        for branch_result in all_branches {
            let (branch, _) = match branch_result {
                Ok(b) => b,
                Err(_) => continue,
            };

            let branch_name = match branch.name() {
                Ok(Some(name)) => name.to_string(),
                _ => continue,
            };

            // 排除当前分支
            if branch_name == current_branch {
                continue;
            }

            let candidate_commit = match branch.get().peel_to_commit() {
                Ok(c) => c,
                Err(_) => continue,
            };

            // 计算 merge base
            let merge_base_oid = match repo.merge_base(current_commit.id(), candidate_commit.id()) {
                Ok(oid) => oid,
                Err(_) => continue,
            };

            // 检查 merge base 是否就是候选分支的 HEAD（精确匹配）
            if merge_base_oid == candidate_commit.id() {
                let timestamp = candidate_commit.time().seconds();
                exact_matches.push((branch_name.clone(), timestamp));
            }

            // 记录主线分支的 merge base 时间（作为备选）
            if MAIN_BRANCHES.contains(&branch_name.as_str()) {
                if let Ok(merge_base_commit) = repo.find_commit(merge_base_oid) {
                    let timestamp = merge_base_commit.time().seconds();
                    main_branch_candidates.push((branch_name, timestamp));
                }
            }
        }

        // 如果有精确匹配，选择 commit 时间最新的（最近的父分支）
        if !exact_matches.is_empty() {
            // 按 commit 时间降序排序
            exact_matches.sort_by(|a, b| b.1.cmp(&a.1));
            return Ok(Some(exact_matches.remove(0).0));
        }

        // 没有精确匹配，选择 merge base 时间最新的主线分支
        if !main_branch_candidates.is_empty() {
            main_branch_candidates.sort_by(|a, b| b.1.cmp(&a.1));
            return Ok(Some(main_branch_candidates.remove(0).0));
        }

        Ok(None)
    }

    /// 从 reflog 消息中提取源分支名
    fn extract_source_branch(message: &str) -> Option<String> {
        // 格式1: "branch: Created from refs/heads/develop"
        if let Some(stripped) = message.strip_prefix("branch: Created from ") {
            let source = stripped.strip_prefix("refs/heads/").unwrap_or(stripped);
            let source = source.trim();

            // 跳过 commit hash（40个十六进制字符）
            if source.len() == 40 && source.chars().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }

            return Some(source.to_string());
        }

        // 格式2: "checkout: moving from develop to feature/new"
        if message.starts_with("checkout: moving from ") {
            if let Some(start) = message.find("from ") {
                if let Some(end) = message.find(" to ") {
                    let from = &message[start + 5..end];
                    return Some(from.trim().to_string());
                }
            }
        }

        // 格式3: "branch: Reset to develop"
        if let Some(stripped) = message.strip_prefix("branch: Reset to ") {
            return Some(stripped.trim().to_string());
        }

        None
    }
}
