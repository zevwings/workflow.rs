//! 设置 Jira 配置命令

use crate::interactive::{core::stage::WorkflowExecutor, platforms::jira::jira_stage};

/// Jira Setup 命令
pub struct JiraSetupCommand;

impl Default for JiraSetupCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl JiraSetupCommand {
    /// 创建新的 JiraSetupCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow jira setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        WorkflowExecutor::new(jira_stage()).run_command_setup()
    }
}
