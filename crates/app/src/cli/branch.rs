//! Branch management subcommands
//!
//! 分支管理子命令结构定义

use clap::Subcommand;

use super::args::{DryRunArgs, ForceArgs, JiraIdArg};

/// Branch management subcommands
///
/// 用于管理分支和分支忽略列表。
#[derive(Subcommand)]
pub enum BranchSubcommand {
    /// 创建新分支
    ///
    /// Create a new branch, optionally from a JIRA ticket.
    Create {
        #[command(flatten)]
        jira_id: JiraIdArg,
        /// 从默认分支（main/master）创建
        /// Create from default branch (main/master)
        #[arg(long)]
        from_default: bool,
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
    /// 切换分支
    ///
    /// Switch to a branch, with interactive selection if branch name is not provided.
    Switch {
        /// 分支名称（可选，不提供时交互式选择）
        /// Branch name (optional, will enter interactive mode if not provided)
        branch_name: Option<String>,
    },
    /// 重命名分支
    ///
    /// Fully interactive branch rename command.
    /// All operations are done through interactive prompts.
    Rename,
    /// 清理本地分支
    ///
    /// Clean local branches (excluding master, develop, current branch, and ignored branches).
    Clean {
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
    /// 管理分支忽略列表
    ///
    /// Manage branch ignore list.
    #[command(subcommand)]
    Ignore(IgnoreSubcommand),
    /// 删除分支
    ///
    /// Remove local and/or remote branch.
    /// If branch name is not provided, will show an interactive list to select branch.
    Remove {
        /// 分支名称（可选，不提供时交互式选择）
        /// Branch name (optional, will enter interactive mode if not provided)
        branch_name: Option<String>,
        /// 只删除本地分支
        /// Remove only local branch
        #[arg(long)]
        local_only: bool,
        /// 只删除远程分支
        /// Remove only remote branch
        #[arg(long)]
        remote_only: bool,
        #[command(flatten)]
        dry_run: DryRunArgs,
        #[command(flatten)]
        force: ForceArgs,
    },
}

/// Branch ignore list management subcommands
///
/// 分支忽略列表管理子命令
#[derive(Subcommand)]
pub enum IgnoreSubcommand {
    /// 添加分支到忽略列表
    /// Add branch to ignore list
    Add {
        /// 分支名称
        /// Branch name to add
        branch_name: String,
    },
    /// 从忽略列表移除分支
    /// Remove branch from ignore list
    Remove {
        /// 分支名称（可选，不提供时交互式多选）
        /// Branch name to remove (optional, will enter interactive multi-select mode if not provided)
        branch_name: Option<String>,
    },
    /// 列出当前仓库的忽略分支
    /// List ignored branches for current repository
    List,
}
