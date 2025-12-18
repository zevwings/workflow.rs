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
/// 支持格式验证，确保输入的 JIRA ID 符合标准格式。
#[derive(Args, Debug, Clone)]
pub struct JiraIdArg {
    /// Jira ticket ID (optional, will prompt interactively if not provided)
    /// Expected format: 'PROJ-123' (ticket) or 'PROJ' (project name)
    #[arg(value_name = "JIRA_ID", value_parser = Self::validate_jira_id)]
    pub jira_id: Option<String>,
}

impl JiraIdArg {
    /// 验证 JIRA ID 格式
    ///
    /// 使用统一的验证逻辑确保 JIRA ID 格式正确。
    ///
    /// # 参数
    ///
    /// * `value` - 待验证的 JIRA ID 字符串
    ///
    /// # 返回
    ///
    /// 成功时返回验证后的字符串，失败时返回格式化的错误信息。
    fn validate_jira_id(value: &str) -> Result<String, String> {
        // 使用现有的验证函数
        match crate::jira::helpers::validate_jira_ticket_format(value) {
            Ok(_) => Ok(value.to_string()),
            Err(e) => Err(Self::format_validation_error(&e.to_string())),
        }
    }

    /// 格式化验证错误消息
    ///
    /// 提供统一的、用户友好的错误消息格式。
    fn format_validation_error(original_error: &str) -> String {
        use crate::base::constants::errors::validation_errors;
        format!(
            "Invalid JIRA ID format.\n{}\n\nError details: {}",
            validation_errors::JIRA_ID_FORMAT_HELP,
            original_error
        )
    }

    /// 获取 JIRA ID（如果存在）
    pub fn get(&self) -> Option<&str> {
        self.jira_id.as_deref()
    }

    /// 获取 JIRA ID（移动所有权）
    pub fn into_option(self) -> Option<String> {
        self.jira_id
    }

    /// 验证并获取 JIRA ID
    ///
    /// 如果提供了 JIRA ID，验证其格式；如果未提供，返回 None。
    /// 这个方法可以用于在运行时进行额外的验证。
    pub fn get_validated(&self) -> Result<Option<String>, String> {
        match &self.jira_id {
            Some(id) => Self::validate_jira_id(id).map(Some),
            None => Ok(None),
        }
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

/// 通用日志级别参数
///
/// 控制输出的详细程度，支持静默模式和详细模式。
#[derive(Args, Debug, Clone)]
pub struct VerbosityArgs {
    /// Verbose output (show detailed information)
    #[arg(long, short = 'v', action = clap::ArgAction::SetTrue)]
    pub verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(long, short = 'q', action = clap::ArgAction::SetTrue)]
    pub quiet: bool,
}

impl VerbosityArgs {
    /// 获取详细模式标志
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// 获取静默模式标志
    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    /// 获取日志级别（优先级：quiet > verbose > normal）
    pub fn get_log_level(&self) -> LogLevel {
        if self.quiet {
            LogLevel::Quiet
        } else if self.verbose {
            LogLevel::Verbose
        } else {
            LogLevel::Normal
        }
    }
}

/// 日志级别枚举
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    /// 静默模式 - 仅显示错误
    Quiet,
    /// 正常模式 - 显示基本信息
    Normal,
    /// 详细模式 - 显示详细信息
    Verbose,
}

/// 通用确认参数
///
/// 用于控制是否跳过交互式确认。
#[derive(Args, Debug, Clone)]
pub struct ConfirmationArgs {
    /// Skip interactive confirmations (assume yes)
    #[arg(long, short = 'y', action = clap::ArgAction::SetTrue)]
    pub yes: bool,
}

impl ConfirmationArgs {
    /// 获取跳过确认标志
    pub fn skip_confirmation(&self) -> bool {
        self.yes
    }
}

// ==================== 组合参数组 ====================

/// 常用的查询和显示参数组合
///
/// 包含输出格式、分页和详细程度控制，适用于大多数查询命令。
#[derive(Args, Debug, Clone)]
pub struct QueryDisplayArgs {
    #[command(flatten)]
    pub output_format: OutputFormatArgs,

    #[command(flatten)]
    pub pagination: PaginationArgs,

    #[command(flatten)]
    pub verbosity: VerbosityArgs,
}

/// 操作执行参数组合
///
/// 包含强制执行、干运行和确认控制，适用于修改性操作。
#[derive(Args, Debug, Clone)]
pub struct OperationArgs {
    #[command(flatten)]
    pub force: ForceArgs,

    #[command(flatten)]
    pub dry_run: DryRunArgs,

    #[command(flatten)]
    pub confirmation: ConfirmationArgs,

    #[command(flatten)]
    pub verbosity: VerbosityArgs,
}

/// Jira 相关操作参数组合
///
/// 包含 JIRA ID 和查询显示参数，适用于 JIRA 相关命令。
#[derive(Args, Debug, Clone)]
pub struct JiraQueryArgs {
    #[command(flatten)]
    pub jira_id: JiraIdArg,

    #[command(flatten)]
    pub query_display: QueryDisplayArgs,
}

/// 带操作控制的 Jira 参数组合
///
/// 包含 JIRA ID 和操作执行参数，适用于 JIRA 修改操作。
#[derive(Args, Debug, Clone)]
pub struct JiraOperationArgs {
    #[command(flatten)]
    pub jira_id: JiraIdArg,

    #[command(flatten)]
    pub operation: OperationArgs,
}
