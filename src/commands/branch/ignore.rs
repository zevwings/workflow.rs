//! 分支忽略列表管理命令
//!
//! 管理分支清理时的忽略列表，支持添加、移除、列出操作。

use crate::commands::branch::{
    add_ignore_branch, get_ignore_branches, remove_ignore_branch, save, BranchConfig,
};
use crate::git::GitRepo;
use crate::{log_break, log_info, log_message, log_success, log_warning};
use anyhow::{Context, Result};

/// 分支忽略列表管理命令
pub struct BranchIgnoreCommand;

impl BranchIgnoreCommand {
    /// 添加分支到忽略列表
    pub fn add(branch_name: String) -> Result<()> {
        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;

        let mut config = BranchConfig::load().context("Failed to load branch config")?;

        let is_new = add_ignore_branch(&mut config, repo_name.clone(), branch_name.clone())?;

        if is_new {
            save(&config).context("Failed to save branch config")?;
            log_success!(
                "已添加分支 '{}' 到忽略列表 (仓库: {})",
                branch_name,
                repo_name
            );
        } else {
            log_info!(
                "分支 '{}' 已在忽略列表中 (仓库: {})",
                branch_name,
                repo_name
            );
        }

        Ok(())
    }

    /// 从忽略列表移除分支
    pub fn remove(branch_name: String) -> Result<()> {
        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;

        let mut config = BranchConfig::load().context("Failed to load branch config")?;

        let removed = remove_ignore_branch(&mut config, &repo_name, &branch_name)?;

        if removed {
            save(&config).context("Failed to save branch config")?;
            log_success!(
                "已从忽略列表移除分支 '{}' (仓库: {})",
                branch_name,
                repo_name
            );
        } else {
            log_warning!(
                "分支 '{}' 不在忽略列表中 (仓库: {})",
                branch_name,
                repo_name
            );
        }

        Ok(())
    }

    /// 列出当前仓库的忽略分支
    pub fn list() -> Result<()> {
        let repo_name =
            GitRepo::extract_repo_name().context("Failed to extract repository name")?;

        let config = BranchConfig::load().context("Failed to load branch config")?;

        let ignore_branches = get_ignore_branches(&config, &repo_name);

        log_break!();
        log_message!("📋 忽略分支列表 (仓库: {})", repo_name);

        if ignore_branches.is_empty() {
            log_info!("当前没有忽略的分支");
        } else {
            for (index, branch) in ignore_branches.iter().enumerate() {
                log_info!("  {}. {}", index + 1, branch);
            }
            log_info!("总计: {} 个分支", ignore_branches.len());
        }

        Ok(())
    }
}
