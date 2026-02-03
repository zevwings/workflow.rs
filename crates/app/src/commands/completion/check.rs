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
        println!("Shell Completion 配置状态:\n");

        // 调用 Service 检查状态
        let service = get_completion_service();
        let result = service.check_status().wrap_err("检查 completion 状态失败")?;

        // 显示当前 shell
        if let Some(ref shell) = result.current_shell {
            println!("当前 Shell: {}\n", shell);
        }

        // 显示 completion 目录
        if let Some(ref dir) = result.completion_dir {
            println!("Completion 目录: {}\n", dir.display());
        }

        // 检查各个 shell 的配置状态
        println!("配置状态:");
        println!("{:-<50}", "");

        for status in &result.shell_statuses {
            // 构建状态标记
            let config_status = if status.is_configured { "✓" } else { "✗" };
            let script_status = if status.script_exists { "✓" } else { "✗" };
            let current_marker = if status.is_current { " (当前)" } else { "" };

            // 显示状态
            println!(
                "  {:<12} 配置: {}  脚本: {}{}",
                status.shell, config_status, script_status, current_marker
            );

            // 显示配置文件路径
            if let Some(ref path) = status.config_file {
                println!("              配置文件: {}", path.display());
            }
        }

        println!("{:-<50}", "");

        // 显示使用提示
        println!("\n提示:");
        println!("  - 使用 'workflow completion generate' 生成并配置 completion");
        println!("  - 使用 'workflow completion remove' 移除 completion 配置");

        Ok(())
    }
}

impl Default for CompletionCheckCommand {
    fn default() -> Self {
        Self::new()
    }
}
