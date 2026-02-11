//! 检查 Jira 配置命令

use prompt::{br, separator};

use crate::interactive::{core::stage::WorkflowExecutor, platforms::jira::jira_stage};

/// Jira Check 命令
pub struct JiraCheckCommand;

impl Default for JiraCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl JiraCheckCommand {
    /// 创建新的 JiraCheckCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow jira check` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('─', 80, "Jira Configuration Check");
        br!();
        WorkflowExecutor::new(jira_stage()).run_verify()
    }
}
