//! 分支业务逻辑服务
//!
//! 提供分支相关的业务逻辑实现。

use super::GitContext;
use domain::git::GitError;
use git2::BranchType;

/// 分支服务接口
pub trait BranchService: Send + Sync {
    /// 创建分支
    fn create_branch(&self, name: &str) -> Result<(), GitError>;

    /// 删除分支
    fn delete_branch(&self, name: &str, force: bool) -> Result<(), GitError>;

    /// 重命名分支
    fn rename_branch(&self, old_name: Option<&str>, new_name: &str) -> Result<(), GitError>;

    /// 获取分支列表
    fn list_branches(
        &self,
        remove_prefix: bool,
        all: bool,
    ) -> Result<Vec<domain::BranchInfo>, GitError>;

    /// 切换分支
    fn checkout_branch(&self, name: &str) -> Result<(), GitError>;

    /// 获取当前分支名
    fn get_current_branch(&self) -> Result<String, GitError>;

    /// 检查分支是否存在 (本地, 远程)
    fn has_branch(&self, name: &str) -> Result<(bool, bool), GitError>;

    /// 获取默认分支
    fn get_default_branch(&self) -> Result<String, GitError>;
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
        let head = repo
            .head()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let commit = head
            .peel_to_commit()
            .map_err(|_| GitError::OperationFailed("无法获取 HEAD 提交，仓库可能为空".into()))?;

        // 创建分支
        repo.branch(name, &commit, false).map_err(|e| {
            if e.code() == git2::ErrorCode::Exists {
                GitError::OperationFailed(format!("分支 '{}' 已存在", name))
            } else {
                GitError::OperationFailed(e.to_string())
            }
        })?;

        Ok(())
    }

    fn delete_branch(&self, name: &str, force: bool) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        let mut branch = repo
            .find_branch(name, BranchType::Local)
            .map_err(|_| GitError::BranchNotFound(name.to_string()))?;

        // 检查是否为当前分支
        if branch.is_head() {
            return Err(GitError::OperationFailed(format!(
                "无法删除当前所在的分支: {}",
                name
            )));
        }

        // 检查是否已合并（如果不强制删除）
        if !force {
            let head = repo
                .head()
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            let head_commit = head
                .peel_to_commit()
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            let branch_commit = branch
                .get()
                .peel_to_commit()
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;

            let is_merged = repo
                .graph_descendant_of(head_commit.id(), branch_commit.id())
                .unwrap_or(false);

            if !is_merged {
                return Err(GitError::BranchNotFullyMerged(name.to_string()));
            }
        }

        branch
            .delete()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    fn rename_branch(&self, old_name: Option<&str>, new_name: &str) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        let mut branch = if let Some(name) = old_name {
            repo.find_branch(name, BranchType::Local)
                .map_err(|_| GitError::BranchNotFound(name.to_string()))?
        } else {
            // 重命名当前分支
            let head = repo
                .head()
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            if !head.is_branch() {
                return Err(GitError::OperationFailed("HEAD 处于 detached 状态".into()));
            }
            let branch_name = head
                .shorthand()
                .ok_or_else(|| GitError::OperationFailed("无效的分支名称".into()))?;
            repo.find_branch(branch_name, BranchType::Local)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?
        };

        branch.rename(new_name, false).map_err(|e| {
            if e.code() == git2::ErrorCode::Exists {
                GitError::OperationFailed(format!("分支 '{}' 已存在", new_name))
            } else {
                GitError::OperationFailed(e.to_string())
            }
        })?;

        Ok(())
    }

    fn list_branches(
        &self,
        remove_prefix: bool,
        all: bool,
    ) -> Result<Vec<domain::BranchInfo>, GitError> {
        let repo = self.ctx.repository();

        use std::collections::HashMap;
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
        let mut result: Vec<domain::BranchInfo> = branches
            .into_iter()
            .map(|(name, (_has_local, has_remote))| {
                let display_name = if has_remote {
                    // 有远程分支，用 * 标记
                    format!("* {}", name)
                } else {
                    // 只有本地分支，用两个空格
                    format!("  {}", name)
                };

                // name 字段保存不带 origin/ 前缀的短名称
                // 这样 has_branch 可以正确检查本地和远程
                let mut info = domain::BranchInfo::local(name.clone());
                info.display_name = display_name;
                info.is_remote = has_remote;
                info
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
                .ok_or_else(|| GitError::OperationFailed("无效的引用名称".into()))?;

            let obj = repo
                .revparse_single(refname)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            repo.checkout_tree(&obj, None)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            repo.set_head(refname)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
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
            let obj = repo
                .revparse_single(&refname)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            repo.checkout_tree(&obj, None)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            repo.set_head(&refname)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        } else {
            // 创建新分支
            self.create_branch(name)?;

            // 切换到新分支
            let refname = format!("refs/heads/{}", name);
            let obj = repo
                .revparse_single(&refname)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            repo.checkout_tree(&obj, None)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            repo.set_head(&refname)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        }

        Ok(())
    }

    fn get_current_branch(&self) -> Result<String, GitError> {
        let repo = self.ctx.repository();

        let head = repo
            .head()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        if !head.is_branch() {
            return Err(GitError::OperationFailed("HEAD 处于 detached 状态".into()));
        }

        head.shorthand()
            .map(String::from)
            .ok_or_else(|| GitError::OperationFailed("无法获取分支名称".into()))
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
                    let short_name = branch_name
                        .strip_prefix("refs/heads/")
                        .unwrap_or(branch_name);
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

        Err(GitError::OperationFailed("无法确定默认分支".into()))
    }
}

impl BranchServiceImpl {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::setup_repo;

    #[test]
    fn test_create_branch() {
        let (_tmp, ctx) = setup_repo();
        let service = BranchServiceImpl::new(ctx);

        service.create_branch("test-branch").unwrap();
        let (exists, _) = service.has_branch("test-branch").unwrap();
        assert!(exists);
    }

    #[test]
    fn test_current_branch() {
        let (_tmp, ctx) = setup_repo();
        let service = BranchServiceImpl::new(ctx);

        let current = service.get_current_branch().unwrap();
        assert!(current == "master" || current == "main");
    }
}
