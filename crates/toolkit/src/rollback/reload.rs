//! Shell 配置重载模块
//!
//! 提供 Shell 配置重新加载功能。

use clap_complete::Shell;
use duct::cmd;
use thiserror::Error;

use crate::shell::config_file_path;

/// Shell 重载错误类型
#[derive(Debug, Error)]
pub enum ReloadError {
    /// 无法获取配置文件路径
    #[error("Failed to get config file path for shell: {0}")]
    ConfigFileNotFound(String),

    /// 重载命令执行失败
    #[error("Failed to reload config: {0}")]
    ReloadFailed(String),
}

/// Shell 重载结果
#[derive(Debug, Clone)]
pub struct ReloadResult {
    /// 是否成功重载
    pub reloaded: bool,
    /// 消息列表
    pub messages: Vec<String>,
    /// 手动重载提示
    pub reload_hint: String,
}

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
    /// # 返回
    ///
    /// 返回 `ReloadResult`，包含重载状态和消息。
    ///
    /// # 错误
    ///
    /// 如果重新加载失败，返回相应的错误信息。
    pub fn shell(shell: &Shell) -> Result<ReloadResult, ReloadError> {
        let config_file = config_file_path(shell)
            .ok_or_else(|| ReloadError::ConfigFileNotFound(shell.to_string()))?;
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
                .map_err(|e| ReloadError::ReloadFailed(e.to_string()))
        } else {
            cmd(&shell_type, &["-c", &shell_cmd])
                .run()
                .map(|_| ())
                .map_err(|e| ReloadError::ReloadFailed(e.to_string()))
        };

        match status {
            Ok(_) => Ok(ReloadResult {
                reloaded: true,
                messages: vec![
                    "Shell configuration reloaded (in subprocess)".to_string(),
                    "Note: Changes may not take effect in the current shell.".to_string(),
                ],
                reload_hint: reload_hint.clone(),
            }),
            Err(e) => {
                tracing::warn!("Could not reload shell configuration: {}", e);
                Ok(ReloadResult {
                    reloaded: false,
                    messages: vec![format!("Could not reload shell configuration: {}", e)],
                    reload_hint: reload_hint.clone(),
                })
            }
        }
    }
}
