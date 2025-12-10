//! 分支前缀管理命令
//!
//! 管理仓库级别的分支前缀配置，支持设置、获取、移除操作。

use crate::commands::branch::{get_branch_prefix, remove_branch_prefix, set_branch_prefix};
use crate::git::GitRepo;
use crate::{log_info, log_success};
use anyhow::{Context, Result};

/// 分支前缀管理命令
pub struct BranchPrefixCommand;

impl BranchPrefixCommand {
    /// 设置当前仓库的分支前缀
    ///
    /// # 参数
    /// * `prefix` - 分支前缀，如果为 None 则通过交互式输入获取
    ///
    /// # 返回
    /// 成功返回 `Ok(())`，失败返回错误
    pub fn set(prefix: Option<String>) -> Result<()> {
        set_branch_prefix(prefix)
    }

    /// 获取当前仓库的分支前缀
    ///
    /// # 返回
    /// 成功返回 `Ok(())`，失败返回错误
    pub fn get() -> Result<()> {
        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;

        if let Some(prefix) = get_branch_prefix() {
            log_success!("Branch prefix for repository '{}': {}", repo_name, prefix);
        } else {
            log_info!("No branch prefix configured for repository: {}", repo_name);
        }

        Ok(())
    }

    /// 移除当前仓库的分支前缀
    ///
    /// # 返回
    /// 成功返回 `Ok(())`，失败返回错误
    pub fn remove() -> Result<()> {
        remove_branch_prefix()
    }
}
