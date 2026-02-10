//! 分支管理子命令
//!
//! 分支管理子命令结构定义

use clap::Subcommand;

use super::super::args::{DryRunArgs, ForceArgs, JiraIdArg};

/// 分支管理子命令
///
/// 用于管理分支和分支忽略列表。
#[derive(Subcommand)]
pub enum BranchSubcommand {
    /// 创建新分支（可选从 JIRA 工单创建）
    Create {
        #[command(flatten)]
        jira_id: JiraIdArg,
        /// 从默认分支（main/master）创建
        #[arg(long)]
        from_default: bool,
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
    /// 切换分支（未提供分支名称时交互式选择）
    Switch {
        /// 分支名称（可选，不提供时交互式选择）
        branch_name: Option<String>,
    },
    /// 重命名分支（完全交互式命令）
    Rename,
    /// 清理本地分支（排除 master、develop、当前分支和忽略的分支）
    Clean {
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
    /// 推断当前分支的源分支（从 reflog 或 merge base）
    #[cfg(feature = "develop")]
    InferSource,
    /// 管理分支忽略列表
    #[command(subcommand)]
    Ignore(IgnoreSubcommand),
    /// 删除分支（未提供分支名称时显示交互式列表选择）
    Remove {
        /// 分支名称（可选，不提供时交互式选择）
        branch_name: Option<String>,
        /// 只删除本地分支
        #[arg(long)]
        local_only: bool,
        /// 只删除远程分支
        #[arg(long)]
        remote_only: bool,
        #[command(flatten)]
        dry_run: DryRunArgs,
        #[command(flatten)]
        force: ForceArgs,
    },
}

/// 分支忽略列表管理子命令
#[derive(Subcommand)]
pub enum IgnoreSubcommand {
    /// 添加分支到忽略列表
    Add {
        /// 分支名称
        branch_name: String,
    },
    /// 从忽略列表移除分支（未提供分支名称时交互式多选）
    Remove {
        /// 分支名称（可选，不提供时交互式多选）
        branch_name: Option<String>,
    },
    /// 列出当前仓库的忽略分支
    List,
}
