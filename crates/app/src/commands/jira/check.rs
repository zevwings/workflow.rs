//! 检查 Jira 配置命令

use prompt::{br, separator};

use crate::bootstrap;
use crate::interactive::{WorkflowExecutor, JIRA_STAGE_NAME};

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
        let stage = bootstrap::get_workflow_stage_registry()
            .stage_by_name(JIRA_STAGE_NAME)
            .expect("Jira stage must be registered");
        WorkflowExecutor::new(stage).run_verify()
    }
}
