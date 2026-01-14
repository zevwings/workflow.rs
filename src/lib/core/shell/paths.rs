//! Shell 路径工具
//!
//! 提供 shell 相关的路径管理功能，使 shell 模块完全独立。

use clap_complete::Shell;
use color_eyre::{eyre::bail, Result};
use std::path::PathBuf;

use super::detect::Detect;

/// 获取用户主目录
///
/// 使用 dirs crate 提供的跨平台主目录获取功能。
///
/// # 返回
///
/// 返回用户主目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法确定主目录，返回错误信息。
pub(crate) fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Cannot determine home directory"))
}

/// 获取 shell 配置文件路径
///
/// 支持的 shell 类型及其配置文件路径：
/// - zsh → `~/.zshrc`
/// - bash → `~/.bash_profile`（如果不存在则使用 `~/.bashrc`）
/// - fish → `~/.config/fish/config.fish`
/// - powershell → `~/.config/powershell/Microsoft.PowerShell_profile.ps1` (Unix) 或
///   `~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1` (Windows)
/// - elvish → `~/.elvish/rc.elv`
///
/// 注意：对于 bash，macOS 通常使用 `.bash_profile`，Linux 使用 `.bashrc`。
/// 此方法会优先使用 `.bash_profile`，如果不存在则使用 `.bashrc`。
///
/// # 参数
///
/// * `shell` - Shell 枚举类型
///
/// # 返回
///
/// 返回 shell 配置文件的 `PathBuf`。
///
/// # 错误
///
/// 如果 `HOME` 环境变量未设置或 shell 类型不支持，返回相应的错误信息。
pub fn config_file(shell: &Shell) -> Result<PathBuf> {
    let home = home_dir()?;

    let config_file = match shell {
        #[cfg(target_os = "windows")]
        Shell::PowerShell => {
            // Windows PowerShell 配置文件路径
            // 优先使用 PowerShell Core 路径
            let pwsh_profile = home
                .join("Documents")
                .join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1");
            let ps_profile = home
                .join("Documents")
                .join("WindowsPowerShell")
                .join("Microsoft.PowerShell_profile.ps1");

            // 如果 PowerShell Core 配置文件存在，使用它；否则使用 Windows PowerShell 路径
            if pwsh_profile.exists() {
                pwsh_profile
            } else {
                ps_profile
            }
        }

        #[cfg(not(target_os = "windows"))]
        Shell::Zsh => home.join(".zshrc"),

        #[cfg(not(target_os = "windows"))]
        Shell::Bash => {
            let bash_profile = home.join(".bash_profile");
            let bashrc = home.join(".bashrc");
            if !bash_profile.exists() && bashrc.exists() {
                bashrc
            } else {
                bash_profile
            }
        }

        #[cfg(not(target_os = "windows"))]
        Shell::Fish => home.join(".config/fish/config.fish"),

        #[cfg(not(target_os = "windows"))]
        Shell::PowerShell => home.join(".config/powershell/Microsoft.PowerShell_profile.ps1"),

        #[cfg(not(target_os = "windows"))]
        Shell::Elvish => home.join(".elvish/rc.elv"),

        _ => bail!("Unsupported shell type"),
    };

    Ok(config_file)
}

/// 获取 shell 配置文件路径（自动检测 shell 类型）
///
/// 使用 `Detect::shell()` 自动检测 shell 类型，并获取对应的配置文件路径。
///
/// # 返回
///
/// 返回 shell 配置文件的路径：
/// - zsh → `~/.zshrc`
/// - bash → `~/.bash_profile`（如果不存在则使用 `~/.bashrc`）
/// - fish → `~/.config/fish/config.fish`
/// - powershell → `~/.config/powershell/Microsoft.PowerShell_profile.ps1`
/// - elvish → `~/.elvish/rc.elv`
///
/// # 错误
///
/// 如果无法检测 shell 类型或获取 HOME 目录，返回相应的错误信息。
pub fn get_config_path() -> Result<PathBuf> {
    let shell = Detect::shell()?;
    config_file(&shell)
}
