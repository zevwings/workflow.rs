//! 设置 Jira 配置命令

use crate::registry;
use crate::workflows::get_jira_id_interactive_optional;
use crate::workflows::utils::jira::ensure_jira_status_config;

/// Jira Setup 命令
pub struct JiraStatusCommand {
    jira_id: Option<String>,
}

impl Default for JiraStatusCommand {
    fn default() -> Self {
        Self::new(None)
    }
}

impl JiraStatusCommand {
    /// 创建新的 PullRequestCreateCommand
    pub fn new(jira_id: Option<String>) -> Self {
        Self { jira_id }
    }

    /// 运行 `workflow jira setup` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let jira_repo = registry::get_jira_repository();
        // 获取 JIRA ID（交互式或从参数）
        let jira_id = get_jira_id_interactive_optional(self.jira_id.clone())?;

        ensure_jira_status_config(jira_repo.as_ref(), &jira_id)?;

        Ok(())
    }
}
