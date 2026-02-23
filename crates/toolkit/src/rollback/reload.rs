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
pub fn reload_shell(shell: &Shell) -> Result<ReloadResult, ReloadError> {
    let config_file = config_file_path(shell)
        .ok_or_else(|| ReloadError::ConfigFileNotFound(shell.to_string()))?;
    let config_file_str = config_file.display().to_string();

    // 构建重载命令：PowerShell 使用 `.`，其他 shell 使用 `source`
    let is_powershell = shell == &Shell::PowerShell;
    let reload_cmd = if is_powershell {
        format!(". {}", config_file_str)
    } else {
        format!("source {}", config_file_str)
    };

    // 在子 shell 中执行重载命令
    let shell_type = shell.to_string();
    let args: &[&str] = if is_powershell {
        &["-NoProfile", "-Command", &reload_cmd]
    } else {
        &["-c", &reload_cmd]
    };

    match cmd(&shell_type, args).run() {
        Ok(_) => Ok(ReloadResult {
            reloaded: true,
            messages: vec![
                "Shell configuration reloaded (in subprocess)".to_string(),
                "Note: Changes may not take effect in the current shell.".to_string(),
            ],
            reload_hint: reload_cmd,
        }),
        Err(e) => {
            tracing::warn!("Could not reload shell configuration: {}", e);
            Ok(ReloadResult {
                reloaded: false,
                messages: vec![format!("Could not reload shell configuration: {}", e)],
                reload_hint: reload_cmd,
            })
        }
    }
}
