//! 环境检查命令实现

use prompt::{br, error, info, separator, spinner, success, warning};

use crate::registry;
use crate::workflows::core::stage::{WorkflowExecutor, WorkflowStage};
use crate::workflows::platforms::{
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

        let config_service = registry::get_global_config_repository();
        let workflow_config_path = toolkit::Paths::workflow_config()?;

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

        // 2. 环境检查
        self.check_git_status()?;
        br!();

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

    /// 检查 Git 仓库状态
    fn check_git_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let repo_info = registry::get_git_repo_repository().get_repo_info();
        if !repo_info.is_valid {
            error!("Not in a Git repository");
            return Err("Not in a Git repository".into());
        }

        let git_repo = registry::get_git_repository();

        // 检查工作区状态（使用 spinner 显示进度）
        let spinner = spinner!("Checking working tree status...").start();
        let status = git_repo.get_working_tree_status();
        spinner.stop();

        let status = status?;

        if status.is_clean() {
            success!("Working tree is clean");
        } else {
            warning!("Working tree has uncommitted changes");
            self.display_uncommitted_files(&status);
            info!("  Consider committing or stashing your changes before proceeding");
        }

        Ok(())
    }

    /// 按状态分组显示未提交的文件
    fn display_uncommitted_files(&self, status: &domain::WorkingTreeStatus) {
        let groups = [
            ("Staged files:", &status.staged),
            ("Unstaged changes:", &status.unstaged),
            ("Untracked files:", &status.untracked),
            ("Conflicted files:", &status.conflicted),
        ];

        for (label, files) in groups {
            if !files.is_empty() {
                info!("  {}:", label);
                for file in files {
                    info!("    - {}", file.path);
                }
            }
        }
    }
}
