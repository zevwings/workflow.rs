//! Commit create 命令实现
//!
//! 使用 GitRepository trait 实现提交功能。

use prompt::{info, success};

use crate::registry;

/// Commit Create 命令
pub struct CommitCreateCommand {
    message: String,
    all: bool,
}

impl CommitCreateCommand {
    /// 创建新的 CommitCreateCommand
    pub fn new(message: String, all: bool) -> Self {
        Self { message, all }
    }

    /// 运行 `workflow commit create` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Repository opened successfully");

        let git_repo = registry::get_git_repository();

        if self.all {
            info!("Adding all files to staging area");
        }

        let oid = git_repo
            .commit(&self.message, self.all)
            .map_err(|e| format!("Failed to create commit: {}", e))?;

        success!("Created commit: {}", oid);
        Ok(())
    }
}
