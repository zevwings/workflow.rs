//! 切换分支命令

use color_eyre::Result;
use prompt::{confirm, error, info, select, success, warning};

use crate::registry;

/// Branch Switch 命令
pub struct BranchSwitchCommand {
    branch_name: Option<String>,
}

impl BranchSwitchCommand {
    /// 创建新的 BranchSwitchCommand
    pub fn new(branch_name: Option<String>) -> Self {
        Self { branch_name }
    }

    /// 运行 `workflow branch switch` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_repo = registry::get_git_repository();

        let target_branch = if let Some(name) = &self.branch_name {
            name.clone()
        } else {
            // 交互式选择分支
            let branch_items = branch_repo
                .list_branches(false, false)
                .map_err(|e| format!("Failed to list branches: {}", e))?;

            if branch_items.is_empty() {
                error!("No branches found");
                return Err("No branches available".into());
            }

            let current_branch = branch_repo
                .get_current_branch()
                .map_err(|e| format!("Failed to get current branch: {}", e))?;

            // 提取分支名称用于选择（本地分支的 name 和 display_name 相同）
            let branch_names: Vec<String> =
                branch_items.iter().map(|item| item.name.clone()).collect();

            // 如果分支数量 > 25，提示使用搜索
            let prompt = if branch_names.len() > 25 {
                format!(
                    "Select branch to switch to ({} branches, use arrow keys and type to search):",
                    branch_names.len()
                )
            } else {
                "Select branch to switch to:".to_string()
            };

            // 找到当前分支的索引作为默认值
            let default_index = branch_names
                .iter()
                .position(|b| b == &current_branch)
                .unwrap_or(0);

            select!(prompt, branch_names)
                .default(default_index)
                .prompt()
                .map_err(|e| format!("Failed to select branch: {}", e))?
        };

        // 检查分支是否存在
        let (exists_local, exists_remote) = branch_repo
            .has_branch(&target_branch)
            .map_err(|e| format!("Failed to check branch existence: {}", e))?;

        if !exists_local && !exists_remote {
            // 分支不存在，询问是否创建
            warning!("Branch '{}' does not exist", target_branch);
            let create = confirm!("Create and switch to new branch '{}'?", target_branch)
                .default(true)
                .prompt()
                .map_err(|e| format!("Failed to get confirmation: {}", e))?;

            if !create {
                info!("Cancelled");
                return Ok(());
            }
        }

        // 切换分支
        info!("Switching to branch '{}'...", target_branch);
        branch_repo
            .checkout_branch(&target_branch)
            .map_err(|e| format!("Failed to switch to branch: {}", e))?;

        success!("Switched to branch '{}'", target_branch);
        Ok(())
    }
}
