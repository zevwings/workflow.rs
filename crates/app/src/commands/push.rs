//! Push 命令实现
//!
//! 最简单的 push 实现，直接使用 git2 库。

use prompt::{info, success};

use storage::git::GitContext;

/// Push 命令
pub struct PushCommand;

impl PushCommand {
    /// 创建新的 PushCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow push` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 打开仓库
        let ctx = GitContext::discover()
            .map_err(|e| format!("Failed to open repository: {}", e))?;

        let repo = ctx.repository();

        // 获取当前分支名
        let head = repo
            .head()
            .map_err(|e| format!("Failed to get HEAD: {}", e))?;

        let branch_name = head
            .shorthand()
            .ok_or("Failed to get branch name")?;

        // 获取远程
        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;

        // 构建 refspec
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

        info!("Pushing to origin/{}...", branch_name);

        // 创建回调
        let callbacks = GitContext::create_callbacks();
        let mut opts = git2::PushOptions::new();
        opts.remote_callbacks(callbacks);

        // 推送
        remote
            .push(&[&refspec], Some(&mut opts))
            .map_err(|e| format!("Failed to push: {}", e))?;

        success!("Successfully pushed to origin/{}", branch_name);
        Ok(())
    }
}
