//! Pull Request 管理子命令结构定义

use clap::Subcommand;

use super::args::{DryRunArgs, ForceArgs, JiraIdArg};

/// Pull Request 管理子命令
#[derive(Subcommand)]
pub enum PrSubcommand {
    /// 创建 Pull Request
    Create {
        #[command(flatten)]
        jira_id: JiraIdArg,
        /// PR 标题
        #[arg(long)]
        title: Option<String>,
        /// PR 描述
        #[arg(long)]
        description: Option<String>,
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
    /// 列出 Pull Requests
    List {
        /// PR 状态过滤（open, closed, all）
        #[arg(long)]
        state: Option<String>,
        /// 限制返回数量
        #[arg(long)]
        limit: Option<usize>,
    },
    /// 添加 Pull Request 评论
    Comment {
        /// PR ID（数字）
        pr_id: String,
        /// 评论内容（可选，不提供时交互式输入）
        comment: Option<String>,
    },
    /// 更新 Pull Request
    Update {
        /// PR ID（数字）
        pr_id: String,
        /// PR 标题
        #[arg(long)]
        title: Option<String>,
        /// PR 正文
        #[arg(long)]
        body: Option<String>,
    },
    /// 合并 Pull Request
    Merge {
        /// PR ID（数字）
        pr_id: String,
        #[command(flatten)]
        force: ForceArgs,
    },
    /// 关闭 Pull Request
    Close {
        /// PR ID（数字）
        pr_id: String,
    },
    /// 批准 Pull Request
    Approve {
        /// PR ID（数字）
        pr_id: String,
    },
    /// 总结 Pull Request
    Summarize {
        /// PR ID（数字，可选，不提供时使用当前分支关联的 PR）
        pr_id: Option<String>,
    },
}
