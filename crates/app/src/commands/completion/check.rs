//! Completion 检查命令
//!
//! 检查 Shell Completion 配置状态。

use color_eyre::{eyre::WrapErr, Result};

use crate::registry::get_completion_service;

/// Completion 检查命令
pub struct CompletionCheckCommand;

impl CompletionCheckCommand {
    /// 创建新的 CompletionCheckCommand 实例
    pub fn new() -> Self {
        Self
    }

    /// 运行检查命令
    pub fn run(&self) -> Result<()> {
        println!("Shell Completion Configuration Status:\n");

        // 调用 Service 检查状态
        let service = get_completion_service();
        let result = service.check_status().wrap_err("Failed to check completion status")?;

        // 显示当前 shell
        if let Some(ref shell) = result.current_shell {
            println!("Current Shell: {}\n", shell);
        }

        // 显示 completion 目录
        if let Some(ref dir) = result.completion_dir {
            println!("Completion Directory: {}\n", dir.display());
        }

        // 检查各个 shell 的配置状态
        println!("Configuration Status:");
        println!("{:-<50}", "");

        for status in &result.shell_statuses {
            // 构建状态标记
            let config_status = if status.is_configured { "✓" } else { "✗" };
            let script_status = if status.script_exists { "✓" } else { "✗" };
            let current_marker = if status.is_current { " (current)" } else { "" };

            // 显示状态
            println!(
                "  {:<12} Config: {}  Script: {}{}",
                status.shell, config_status, script_status, current_marker
            );

            // 显示配置文件路径
            if let Some(ref path) = status.config_file {
                println!("              Config file: {}", path.display());
            }
        }

        println!("{:-<50}", "");

        // 显示使用提示
        println!("\nHints:");
        println!("  - Use 'workflow completion generate' to generate and configure completion");
        println!("  - Use 'workflow completion remove' to remove completion config");

        Ok(())
    }
}

impl Default for CompletionCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}
