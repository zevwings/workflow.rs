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
