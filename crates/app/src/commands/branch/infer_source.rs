//! 推断当前分支的源分支命令

use prompt::{info, success};

use crate::registry;

/// Branch InferSource 命令
#[derive(Default)]
pub struct BranchInferSourceCommand;

impl BranchInferSourceCommand {
    /// 创建新的 BranchInferSourceCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow branch infer-source` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_repo = registry::get_git_repository();

        let current_branch = branch_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        let source = branch_repo
            .infer_target_branch(&current_branch)
            .map_err(|e| format!("Failed to infer source branch: {}", e))?;

        match source {
            Some(name) => {
                success!("Source branch of '{}': {}", current_branch, name);
            }
            None => {
                info!("Cannot infer source branch for '{}'", current_branch);
            }
        }
        Ok(())
    }
}
