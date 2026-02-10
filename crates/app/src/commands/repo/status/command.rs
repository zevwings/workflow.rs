//! 仓库状态命令
//!
//! 查看当前仓库状态。

use domain::WorkingTreeStatus;
use prompt::{br, error, info, spinner, success, warning};

use crate::bootstrap::get_git_repository;

/// Repo Status 命令
pub struct RepoStatusCommand;

impl Default for RepoStatusCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl RepoStatusCommand {
    /// 创建新的 RepoStatusCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行仓库状态检查
    ///
    /// 此方法可以被：
    /// 1. 用户直接调用：`workflow repo status`
    /// 2. 其他命令调用：`repo::status::RepoStatusCommand::new().run()`
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting check repository status ...");
        // 2. 环境检查
        self.check_git_status()?;
        br!();

        Ok(())
    }

    /// 检查 Git 仓库状态
    fn check_git_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let git_repo = get_git_repository();

        let repo_info = git_repo.get_repo_info();
        if !repo_info.is_valid {
            error!("Not in a Git repository");
            return Err("Not in a Git repository".into());
        }

        // 检查工作区状态（使用 spinner 显示进度）
        let spinner = spinner!("Checking working tree status...").start();
        let status = git_repo.get_working_tree_status();
        spinner.stop();

        let status = status?;

        if status.is_clean() {
            success!("Working tree is clean");
        } else {
            warning!("Working tree has uncommitted changes");
            self.display_uncommitted_files(&status);
            info!("  Consider committing or stashing your changes before proceeding");
        }

        Ok(())
    }

    /// 按状态分组显示未提交的文件
    fn display_uncommitted_files(&self, status: &WorkingTreeStatus) {
        let groups = [
            ("Staged files:", &status.staged),
            ("Unstaged changes:", &status.unstaged),
            ("Untracked files:", &status.untracked),
            ("Conflicted files:", &status.conflicted),
        ];

        for (label, files) in groups {
            if !files.is_empty() {
                info!("  {}:", label);
                for file in files {
                    info!("    - {}", file.path);
                }
            }
        }
    }
}
