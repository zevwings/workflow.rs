//! 共用 CLI 参数定义
//!
//! 提供多个命令共享的参数组，减少代码重复。
//!
//! 使用 clap 的 `Args` trait 和 `#[command(flatten)]` 特性来实现参数复用。

use clap::Args;
use domain::validate_jira_ticket_format;

/// Dry run 模式选项
///
/// 预览操作而不实际执行。
#[derive(Args, Debug, Clone)]
pub struct DryRunArgs {
    /// Dry run 模式（预览变更但不实际执行）
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
    /// Jira 工单 ID（可选，未提供时将交互式提示）
    /// 期望格式：'PROJ-123'（工单）或 'PROJ'（项目名称）
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
        match validate_jira_ticket_format(value) {
            Ok(_) => Ok(value.to_string()),
            Err(e) => Err(format!(
                "无效的 JIRA ID 格式: {}\n\n期望格式:\n  • 工单 ID: PROJ-123（项目代码 + 连字符 + 数字）\n  • 项目名称: PROJ（仅字母、数字、下划线）",
                e
            )),
        }
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

/// 强制执行参数
///
/// 用于跳过确认和检查，强制执行操作。
#[derive(Args, Debug, Clone)]
pub struct ForceArgs {
    /// 强制执行操作（跳过检查和确认）
    #[arg(long, short = 'f', action = clap::ArgAction::SetTrue)]
    pub force: bool,
}

impl ForceArgs {
    /// 获取 force 标志
    pub fn is_force(&self) -> bool {
        self.force
    }
}
