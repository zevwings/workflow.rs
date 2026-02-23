//! 设置 Jira 配置命令

use crate::bootstrap;
use crate::interactive::{WorkflowExecutor, JIRA_STAGE_NAME};

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
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(JIRA_STAGE_NAME)
            .expect("Jira stage must be registered");
        WorkflowExecutor::new(stage).run_command_setup()
    }
}
