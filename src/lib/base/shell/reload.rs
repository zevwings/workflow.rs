use anyhow::Result;
use clap_complete::Shell;
use duct::cmd;

use crate::base::settings::paths::Paths;
use crate::{log_info, log_success, log_warning};

/// Shell 配置重载工具
///
/// 提供 Shell 配置重新加载功能。
pub struct Reload;

impl Reload {
    /// 重新加载 shell 配置（在子进程中执行 source 命令）
    ///
    /// 在子 shell 中执行 `source` 命令（Unix）或 `.` 命令（PowerShell）来重新加载配置文件。
    /// 注意：这不会影响当前 shell，但可以验证配置文件是否有效。
    ///
    /// # 参数
    ///
    /// * `shell` - Shell 类型
    ///
    /// # 错误
    ///
    /// 如果重新加载失败，返回相应的错误信息。
    pub fn shell(shell: &Shell) -> Result<()> {
        let config_file = Paths::config_file(shell)?;
        let config_file_str = config_file.display().to_string();

        // PowerShell 使用 `.` 而不是 `source`
        let (shell_cmd, reload_hint) = if shell == &Shell::PowerShell {
            (
                format!(". {}", config_file_str),
                format!(". {}", config_file_str),
            )
        } else {
            (
                format!("source {}", config_file_str),
                format!("source {}", config_file_str),
            )
        };

        // 尝试在子 shell 中执行 source 命令
        // 注意：这不会影响当前 shell，但可以验证配置文件是否有效
        let shell_type = shell.to_string();

        // PowerShell 使用不同的参数格式
        let status = if shell == &Shell::PowerShell {
            cmd(&shell_type, &["-NoProfile", "-Command", &shell_cmd])
                .run()
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!("Failed to reload config: {}", e))
        } else {
            cmd(&shell_type, &["-c", &shell_cmd])
                .run()
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!("Failed to reload config: {}", e))
        };

        match status {
            Ok(_) => {
                log_success!("Shell configuration reloaded (in subprocess)");
                log_info!("Note: Changes may not take effect in the current shell.");
                log_info!("Please run manually: {}", reload_hint);
                Ok(())
            }
            Err(e) => {
                log_warning!("  Could not reload shell configuration: {}", e);
                log_info!("Please run manually: {}", reload_hint);
                Err(e)
            }
        }
    }
}
