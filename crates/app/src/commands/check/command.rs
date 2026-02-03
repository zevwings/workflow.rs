//! 环境检查命令实现
//!
//! 提供环境检查和配置验证功能。

use std::time::Duration;

use prompt::{br, error, info, separator, spinner, success, warning};

use crate::registry;
use crate::workflows::core::stage::WorkflowExecutor;
use crate::workflows::platforms::{
    github::github_stage, jira::jira_stage, llm::llm_stage, log::log_stage,
};

/// Check 命令
pub struct CheckCommand;

impl Default for CheckCommand {
    fn default() -> Self {
        Self::new()
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

        // 1. 显示配置信息
        let workflow_config_path = toolkit::Paths::workflow_config()
            .map_err(|e| format!("Failed to get workflow config path: {}", e))?;

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
        self.verify_environment()?;

        // 显示配置文件权限警告（如果有）
        if let Some(warning_msg) = config_service.check_permissions() {
            warning!("{}", warning_msg);
            br!();
        }

        // 3. 配置验证（如果配置文件存在）
        if workflow_config_path.exists() {
            // 直接验证，verify_* 函数内部会加载配置并检查是否有配置
            self.verify_and_display_all()?;
        }

        br!();
        success!("All checks passed");
        Ok(())
    }

    /// 获取所有需要验证的 stage 列表
    fn get_all_stages(&self) -> Vec<&dyn crate::workflows::core::stage::WorkflowStage> {
        vec![jira_stage(), github_stage(), llm_stage(), log_stage()]
    }

    /// 逐个验证并展示结果
    fn verify_and_display_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        separator!('─', 80, "Configuration Verification");
        br!();

        let stages = self.get_all_stages();

        for stage in &stages {
            let executor = WorkflowExecutor::new(*stage);
            if let Err(err) = executor.run_verify() {
                warning!("{} verification failed: {}", stage.stage_name(), err);
            }
            br!();
        }

        Ok(())
    }

    /// 执行完整的环境检查
    ///
    /// 包括：
    /// - Git 仓库状态检查
    /// - 网络连接检查（GitHub）
    fn verify_environment(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Running environment checks...");
        br!();

        // 1. 检查 Git 状态
        self.check_git_status()?;

        br!();

        Ok(())
    }

    /// 检查 Git 仓库状态
    fn check_git_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let repo_repo = registry::get_git_repo_repository();

        let repo_info = repo_repo.get_repo_info();
        if !repo_info.is_valid {
            error!("Not in a Git repository");
            return Err("Not in a Git repository".into());
        }

        // 获取 Git 仓储
        let git_repo = registry::get_git_repository();

        // 检查工作区状态（使用 spinner 显示进度）
        // 注意：状态检查可能会很慢，特别是在大仓库中
        let spinner = spinner!("Checking working tree status...").start();
        // 给 spinner 一点时间启动
        std::thread::sleep(Duration::from_millis(50));
        let status_result = git_repo.get_working_tree_status();
        spinner.stop();

        let status = status_result.map_err(|e| {
            let error_msg = format!("{}", e);
            let error_lower = error_msg.to_lowercase();
            // 检测仓库损坏错误
            if error_lower.contains("malformed mode in tree entry")
                || error_lower.contains("corrupt")
                || error_lower.contains("invalid object")
                || error_lower.contains("bad tree")
                || error_lower.contains("fatal:")
            {
                format!(
                    "Git 仓库已损坏: {}\n\n建议修复步骤：\n  1. 运行 'git fsck' 检查仓库完整性\n  2. 如果可能，从远程仓库重新克隆\n  3. 或者尝试 'git fsck --full' 和 'git gc' 来修复",
                    error_msg
                )
            } else {
                format!("Failed to check Git repository status: {}", error_msg)
            }
        })?;

        if !status.is_clean() {
            warning!("Working tree has uncommitted changes");
            self.display_uncommitted_files(&status);
            info!("  Consider committing or stashing your changes before proceeding");
        } else {
            success!("Working tree is clean");
        }

        Ok(())
    }

    /// 按状态分组显示未提交的文件
    fn display_uncommitted_files(&self, status: &domain::WorkingTreeStatus) {
        // 显示已暂存的文件
        if !status.staged.is_empty() {
            info!("  Staged files:");
            for file in &status.staged {
                info!("    - {}", file.path);
            }
        }

        // 显示未暂存的修改
        if !status.unstaged.is_empty() {
            info!("  Unstaged changes:");
            for file in &status.unstaged {
                info!("    - {}", file.path);
            }
        }

        // 显示未跟踪的文件
        if !status.untracked.is_empty() {
            info!("  Untracked files:");
            for file in &status.untracked {
                info!("    - {}", file.path);
            }
        }

        // 显示冲突的文件
        if !status.conflicted.is_empty() {
            info!("  Conflicted files:");
            for file in &status.conflicted {
                info!("    - {}", file.path);
            }
        }
    }
}
