//! 环境检查命令实现

use prompt::{br, info, separator, success, warning};

use crate::bootstrap::{get_global_config_repository, get_path_service};

use crate::interactive::core::stage::{WorkflowExecutor, WorkflowStage};
use crate::interactive::platforms::{
    github::github_stage, jira::jira_stage, llm::llm_stage, log::log_stage,
};

/// 所有需要验证的 stages
const STAGES: &[fn() -> &'static dyn WorkflowStage] =
    &[jira_stage, github_stage, llm_stage, log_stage];

/// Check 命令
pub struct CheckCommand;

impl Default for CheckCommand {
    fn default() -> Self {
        Self
    }
}

impl CheckCommand {
    /// 创建新的 CheckCommand
    pub fn new() -> Self {
        Self
    }

    /// 运行 `workflow check` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting check command");
        br!();

        let path_service = get_path_service();
        let config_service = get_global_config_repository();
        let workflow_config_path = path_service.get_workflow_config_filepath()?;

        // 1. 显示配置信息
        if workflow_config_path.exists() {
            separator!('=', 80, "Current Configuration");
            br!();
            info!("Workflow config: {:?}", workflow_config_path);
            br!();
        } else {
            warning!("Config file not found");
            br!();
        }

        // 显示配置文件权限警告（如果有）
        if let Some(warning_msg) = config_service.check_permissions() {
            warning!("{}", warning_msg);
            br!();
        }

        // 3. 配置验证（如果配置文件存在）
        if workflow_config_path.exists() {
            self.verify_stages();
        }

        br!();
        success!("All checks passed");
        Ok(())
    }

    /// 验证所有 stages
    fn verify_stages(&self) {
        separator!('─', 80, "Configuration Verification");
        br!();

        for stage_fn in STAGES {
            let stage = stage_fn();
            let executor = WorkflowExecutor::new(stage);
            if let Err(err) = executor.run_verify() {
                warning!("{} verification failed: {}", stage.stage_name(), err);
            }
            br!();
        }
    }
}
