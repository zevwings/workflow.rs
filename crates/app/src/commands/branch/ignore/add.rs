//! 添加分支到忽略列表

use crate::bootstrap;
use prompt::{info, success};

/// Branch Ignore Add 命令
pub struct BranchIgnoreAddCommand {
    branch_name: String,
}

impl BranchIgnoreAddCommand {
    /// 创建新的 BranchIgnoreAddCommand
    pub fn new(branch_name: String) -> Self {
        Self { branch_name }
    }

    /// 添加分支到忽略列表
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let repo = bootstrap::get_repo_config_repository();

        let mut user_config = repo
            .load_user_config()
            .map_err(|e| format!("Failed to load user config: {}", e))
            .unwrap_or_default();

        // 检查分支是否已在忽略列表中
        if user_config.branch.ignore.contains(&self.branch_name) {
            info!(
                "Branch '{}' is already in the ignore list",
                self.branch_name
            );
            return Ok(());
        }

        // 添加到忽略列表
        user_config.branch.ignore.push(self.branch_name.clone());
        user_config.branch.ignore.sort();
        user_config.branch.ignore.dedup();

        // 保存配置
        repo.save_user_config(&user_config)
            .map_err(|e| format!("Failed to save user config: {}", e))?;

        success!("Added branch '{}' to ignore list", self.branch_name);
        Ok(())
    }
}
