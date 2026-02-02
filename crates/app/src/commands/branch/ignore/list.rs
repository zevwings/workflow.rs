//! 列出忽略分支

use crate::registry;
use color_eyre::Result;
use prompt::info;

/// Branch Ignore List 命令
pub struct BranchIgnoreListCommand;

impl Default for BranchIgnoreListCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl BranchIgnoreListCommand {
    /// 创建新的 BranchIgnoreListCommand
    pub fn new() -> Self {
        Self
    }

    /// 列出当前仓库的忽略分支
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let repo = registry::get_repo_config_repository();

        let user_config = repo
            .load_user_config()
            .map_err(|e| format!("Failed to load user config: {}", e))
            .unwrap_or_default();

        let ignore_list = &user_config.branch.ignore;

        if ignore_list.is_empty() {
            info!("No branches in ignore list");
            return Ok(());
        }

        info!("Ignored branches ({}):", ignore_list.len());
        for branch in ignore_list {
            info!("  - {}", branch);
        }

        Ok(())
    }
}
