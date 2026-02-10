//! 从忽略列表移除分支

use crate::bootstrap;
use prompt::{info, success, MultiSelectBuilder};

/// Branch Ignore Remove 命令
pub struct BranchIgnoreRemoveCommand {
    branch_name: Option<String>,
}

impl BranchIgnoreRemoveCommand {
    /// 创建新的 BranchIgnoreRemoveCommand
    pub fn new(branch_name: Option<String>) -> Self {
        Self { branch_name }
    }

    /// 从忽略列表移除分支
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let repo = bootstrap::get_repo_config_repository();

        let mut user_config = repo
            .load_user_config()
            .map_err(|e| format!("Failed to load user config: {}", e))
            .unwrap_or_default();

        // 如果没有提供分支名称，使用交互式多选
        let branches_to_remove = if let Some(name) = &self.branch_name {
            vec![name.clone()]
        } else {
            // 如果忽略列表为空，提示并返回
            if user_config.branch.ignore.is_empty() {
                info!("No branches in ignore list");
                return Ok(());
            }

            // 克隆忽略列表用于多选
            let ignore_list = user_config.branch.ignore.clone();

            // 使用多选框让用户选择要移除的分支
            let selected =
                MultiSelectBuilder::new("Select branches to remove from ignore list", ignore_list)
                    .result_title("Branches to remove")
                    .prompt()
                    .map_err(|e| format!("Failed to prompt for branch selection: {}", e))?;

            if selected.is_empty() {
                info!("No branches selected for removal");
                return Ok(());
            }

            selected
        };

        // 移除选中的分支
        let mut removed_count = 0;
        for branch_name in &branches_to_remove {
            let was_present = user_config.branch.ignore.contains(branch_name);
            if was_present {
                user_config.branch.ignore.retain(|b| b != branch_name);
                removed_count += 1;
            } else {
                info!("Branch '{}' is not in the ignore list", branch_name);
            }
        }

        // 如果有分支被移除，保存配置
        if removed_count > 0 {
            repo.save_user_config(&user_config)
                .map_err(|e| format!("Failed to save user config: {}", e))?;

            if removed_count == 1 {
                success!(
                    "Removed branch '{}' from ignore list",
                    branches_to_remove[0]
                );
            } else {
                success!("Removed {} branches from ignore list", removed_count);
            }
        }

        Ok(())
    }
}
