//! Git 操作模块
//! 包含 Git 命令封装、仓库类型检测等功能

mod commands;
mod repo;
mod types;

// 重新导出所有公共 API
pub use commands::Git;
pub use repo::{detect_repo_type, get_remote_url, has_uncommitted_changes, is_git_repo};
pub use types::RepoType;

// 为了向后兼容，在 Git 结构体上添加便捷方法
impl Git {
    /// 检测远程仓库类型（GitHub 或 Codeup）
    pub fn detect_repo_type() -> anyhow::Result<RepoType> {
        repo::detect_repo_type()
    }

    /// 检查是否在 Git 仓库中
    pub fn is_git_repo() -> bool {
        repo::is_git_repo()
    }

    /// 获取远程仓库 URL
    #[allow(dead_code)]
    pub fn get_remote_url() -> anyhow::Result<String> {
        repo::get_remote_url()
    }

    /// 检查工作区是否有未提交的更改
    #[allow(dead_code)]
    pub fn has_uncommitted_changes() -> anyhow::Result<bool> {
        repo::has_uncommitted_changes()
    }
}
