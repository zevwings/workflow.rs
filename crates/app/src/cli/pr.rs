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
    /// 提交本地更改并推送到 PR
    ///
    /// 基于暂存区内容用 AI 生成 commit message，
    /// 若有 Jira key 则以「jira_key: ai_message」形式提交并推送。
    Update {
        /// PR ID（数字，可选，不提供时使用当前分支关联的 PR）
        pr_id: Option<String>,
        /// 自定义 commit message（可选，不提供时由 AI 根据暂存内容生成）
        #[arg(long, short)]
        message: Option<String>,
        #[command(flatten)]
        dry_run: DryRunArgs,
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
    /// 基于三阶段提交分析重新生成 PR 描述并更新到远端
    Reword {
        #[command(flatten)]
        dry_run: DryRunArgs,
    },
}
