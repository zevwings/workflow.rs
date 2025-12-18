//! 共用 CLI 参数定义
//!
//! 提供多个命令共享的参数组，减少代码重复。
//!
//! 使用 clap 的 `Args` trait 和 `#[command(flatten)]` 特性来实现参数复用。

use clap::Args;

/// 输出格式选项
///
/// 支持多种输出格式：table（默认）、json、yaml、markdown。
/// 优先级：json > yaml > markdown > table
#[derive(Args, Debug, Clone)]
pub struct OutputFormatArgs {
    /// Output in table format (default)
    #[arg(long)]
    pub table: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Output in YAML format
    #[arg(long)]
    pub yaml: bool,

    /// Output in Markdown format
    #[arg(long)]
    pub markdown: bool,
}

/// Dry run 模式选项
///
/// 预览操作而不实际执行。
#[derive(Args, Debug, Clone)]
pub struct DryRunArgs {
    /// Dry run mode (preview changes without actually executing)
    #[arg(long, short = 'n', action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,
}

impl DryRunArgs {
    /// 获取 dry_run 标志
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }
}

/// 可选 JIRA ID 参数
///
/// JIRA ticket ID，如果未提供则交互式输入。
#[derive(Args, Debug, Clone)]
pub struct JiraIdArg {
    /// Jira ticket ID (optional, will prompt interactively if not provided)
    #[arg(value_name = "JIRA_ID")]
    pub jira_id: Option<String>,
}

impl JiraIdArg {
    /// 获取 JIRA ID（如果存在）
    pub fn get(&self) -> Option<&str> {
        self.jira_id.as_deref()
    }

    /// 获取 JIRA ID（移动所有权）
    pub fn into_option(self) -> Option<String> {
        self.jira_id
    }
}

/// 分页参数
///
/// 用于控制结果的分页显示，支持限制结果数量和偏移量。
#[derive(Args, Debug, Clone)]
pub struct PaginationArgs {
    /// Limit number of results to display
    #[arg(long, value_name = "LIMIT")]
    pub limit: Option<usize>,

    /// Offset for pagination
    #[arg(long, value_name = "OFFSET")]
    pub offset: Option<usize>,
}

impl PaginationArgs {
    /// 获取 limit 值
    pub fn get_limit(&self) -> Option<usize> {
        self.limit
    }

    /// 获取 offset 值
    pub fn get_offset(&self) -> Option<usize> {
        self.offset
    }
}

/// 强制执行参数
///
/// 用于跳过确认和检查，强制执行操作。
#[derive(Args, Debug, Clone)]
pub struct ForceArgs {
    /// Force operation (skip checks and confirmations)
    #[arg(long, short = 'f', action = clap::ArgAction::SetTrue)]
    pub force: bool,
}

impl ForceArgs {
    /// 获取 force 标志
    pub fn is_force(&self) -> bool {
        self.force
    }
}
