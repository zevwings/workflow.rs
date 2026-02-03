//! Shell 检测和配置管理模块
//!
//! 本模块提供了 Shell 检测和配置管理功能，包括：
//! - 检测当前 Shell 类型
//! - 获取 Shell 配置文件路径
//! - 提供 Shell 重载提示

pub mod config;
mod error;

use std::path::PathBuf;

use clap_complete::Shell;

pub use config::ShellConfigManager;
pub use error::ShellError;

/// 检测当前 Shell 类型
///
/// 使用 `whattheshell` 库通过进程树推断当前 shell 类型，
/// 然后转换为 `clap_complete::Shell` 枚举。
///
/// # 返回
///
/// 返回检测到的 Shell 类型，如果无法检测则返回错误。
///
/// # 示例
///
/// ```rust,no_run
/// use toolkit::shell::detect_shell;
///
/// match detect_shell() {
///     Ok(shell) => println!("当前 Shell: {:?}", shell),
///     Err(e) => eprintln!("无法检测 Shell: {}", e),
/// }
/// ```
pub fn detect_shell() -> Result<Shell, ShellError> {
    let shell = whattheshell::Shell::infer().map_err(|_| ShellError::DetectionFailed)?;

    match shell {
        whattheshell::Shell::Zsh => Ok(Shell::Zsh),
        whattheshell::Shell::Bash => Ok(Shell::Bash),
        whattheshell::Shell::Fish => Ok(Shell::Fish),
        whattheshell::Shell::PowerShell => Ok(Shell::PowerShell),
        _ => Err(ShellError::UnsupportedShell(shell.to_string())),
    }
}

/// 获取 Shell 配置文件路径
///
/// 根据 Shell 类型返回对应的配置文件路径。
///
/// # 参数
///
/// * `shell` - Shell 类型
///
/// # 返回
///
/// 返回配置文件路径，如果无法获取则返回 None。
///
/// # 路径映射
///
/// - Zsh: `~/.zshrc`
/// - Bash: `~/.bash_profile` (macOS) 或 `~/.bashrc` (Linux)
/// - Fish: `~/.config/fish/config.fish`
/// - PowerShell: `~/.config/powershell/Microsoft.PowerShell_profile.ps1`
/// - Elvish: `~/.config/elvish/rc.elv`
pub fn config_file_path(shell: &Shell) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    Some(match shell {
        Shell::Zsh => home.join(".zshrc"),
        Shell::Bash => {
            // macOS 默认使用 .bash_profile，Linux 使用 .bashrc
            #[cfg(target_os = "macos")]
            {
                home.join(".bash_profile")
            }
            #[cfg(not(target_os = "macos"))]
            {
                home.join(".bashrc")
            }
        }
        Shell::Fish => home.join(".config/fish/config.fish"),
        Shell::PowerShell => {
            home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
        }
        Shell::Elvish => home.join(".config/elvish/rc.elv"),
        _ => return None,
    })
}

/// 获取 Shell 重载命令提示
///
/// 返回用于重新加载 Shell 配置的命令提示。
///
/// # 参数
///
/// * `shell` - Shell 类型
///
/// # 返回
///
/// 返回重载命令字符串。
pub fn reload_hint(shell: &Shell) -> &'static str {
    match shell {
        Shell::Zsh => "source ~/.zshrc",
        Shell::Bash => {
            #[cfg(target_os = "macos")]
            {
                "source ~/.bash_profile"
            }
            #[cfg(not(target_os = "macos"))]
            {
                "source ~/.bashrc"
            }
        }
        Shell::Fish => "source ~/.config/fish/config.fish",
        Shell::PowerShell => ". $PROFILE",
        Shell::Elvish => "exec elvish",
        _ => "Please restart your terminal",
    }
}

/// 将 Shell 类型转换为字符串
///
/// 返回 Shell 类型的小写字符串表示。
pub fn shell_to_string(shell: &Shell) -> &'static str {
    match shell {
        Shell::Zsh => "zsh",
        Shell::Bash => "bash",
        Shell::Fish => "fish",
        Shell::PowerShell => "powershell",
        Shell::Elvish => "elvish",
        _ => "unknown",
    }
}

/// 从字符串解析 Shell 类型
///
/// # 参数
///
/// * `s` - Shell 类型字符串
///
/// # 返回
///
/// 返回解析后的 Shell 类型，如果无法解析则返回错误。
pub fn shell_from_string(s: &str) -> Result<Shell, ShellError> {
    match s.to_lowercase().as_str() {
        "zsh" => Ok(Shell::Zsh),
        "bash" => Ok(Shell::Bash),
        "fish" => Ok(Shell::Fish),
        "powershell" | "pwsh" => Ok(Shell::PowerShell),
        "elvish" => Ok(Shell::Elvish),
        _ => Err(ShellError::UnsupportedShell(s.to_string())),
    }
}

/// 获取所有支持的 Shell 类型列表
pub fn supported_shells() -> Vec<Shell> {
    vec![
        Shell::Zsh,
        Shell::Bash,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ]
}
