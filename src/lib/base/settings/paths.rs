//! 路径管理
//!
//! 统一管理所有路径信息，包括：
//! - 配置文件路径（存储在 `~/.workflow/config/` 目录下）
//! - 安装路径（二进制文件和补全脚本的安装路径和名称）
//! - Shell 相关路径（shell 配置文件和 completion 目录）

use anyhow::{Context, Result};
use clap_complete::shells::Shell;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 路径管理器
///
/// 统一管理所有路径信息，包括配置路径、安装路径和 Shell 路径。
pub struct Paths;

impl Paths {
    // ==================== 配置路径相关方法 ====================

    /// 获取配置目录路径
    ///
    /// 返回 `~/.workflow/config/` 目录路径，如果目录不存在则创建。
    ///
    /// # 返回
    ///
    /// 返回配置目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置或无法创建目录，返回相应的错误信息。
    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let config_dir = PathBuf::from(home).join(".workflow").join("config");

        // 确保配置目录存在
        fs::create_dir_all(&config_dir).context("Failed to create .workflow/config directory")?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set config directory permissions")?;
        }

        Ok(config_dir)
    }

    /// 获取主配置文件路径
    ///
    /// 返回 `~/.workflow/config/workflow.toml` 的路径。
    pub fn workflow_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("workflow.toml"))
    }

    /// 获取 LLM 配置文件路径
    ///
    /// 返回 `~/.workflow/config/llm.toml` 的路径。
    pub fn llm_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("llm.toml"))
    }

    /// 获取 Jira 状态配置文件路径
    ///
    /// 返回 `~/.workflow/config/jira-status.toml` 的路径。
    pub fn jira_status_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("jira-status.toml"))
    }

    /// 获取 Jira 用户配置文件路径
    ///
    /// 返回 `~/.workflow/config/jira-users.toml` 的路径。
    pub fn jira_users_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("jira-users.toml"))
    }

    /// 获取分支配置文件路径
    ///
    /// 返回 `~/.workflow/config/branch.toml` 的路径。
    pub fn branch_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("branch.toml"))
    }

    /// 获取工作流目录路径
    ///
    /// 返回 `~/.workflow/` 目录路径，如果目录不存在则创建。
    ///
    /// # 返回
    ///
    /// 返回工作流目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置或无法创建目录，返回相应的错误信息。
    pub fn workflow_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_dir = PathBuf::from(home).join(".workflow");

        // 确保工作流目录存在
        fs::create_dir_all(&workflow_dir).context("Failed to create .workflow directory")?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set workflow directory permissions")?;
        }

        Ok(workflow_dir)
    }

    /// 获取工作历史记录目录路径
    ///
    /// 返回 `~/.workflow/work-history/` 目录路径，如果目录不存在则创建。
    ///
    /// # 返回
    ///
    /// 返回工作历史记录目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置或无法创建目录，返回相应的错误信息。
    pub fn work_history_dir() -> Result<PathBuf> {
        let history_dir = Self::workflow_dir()?.join("work-history");

        // 确保目录存在
        fs::create_dir_all(&history_dir)
            .context("Failed to create .workflow/work-history directory")?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&history_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set work-history directory permissions")?;
        }

        Ok(history_dir)
    }

    // ==================== 安装路径相关方法 ====================
    /// 获取所有命令名称
    ///
    /// 返回所有 Workflow CLI 命令的名称列表，这些名称同时用于：
    /// - 二进制文件名（workflow）
    /// - 补全脚本命令名（用于生成补全脚本）
    ///
    /// # 返回
    ///
    /// 返回包含所有命令名称的静态字符串切片数组。
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::settings::paths::Paths;
    ///
    /// let names = Paths::command_names();
    /// assert_eq!(names, ["workflow"]);
    /// ```
    pub fn command_names() -> &'static [&'static str] {
        &["workflow"]
    }

    /// 获取二进制文件安装目录
    ///
    /// 返回二进制文件安装的系统目录路径。
    ///
    /// # 返回
    ///
    /// 返回安装目录路径的静态字符串。
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::settings::paths::Paths;
    ///
    /// let dir = Paths::binary_install_dir();
    /// assert_eq!(dir, "/usr/local/bin");
    /// ```
    pub fn binary_install_dir() -> &'static str {
        "/usr/local/bin"
    }

    /// 获取所有二进制文件的完整路径
    ///
    /// 基于 `command_names()` 和 `binary_install_dir()` 构建完整路径。
    ///
    /// # 返回
    ///
    /// 返回包含所有二进制文件完整路径的字符串向量。
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::settings::paths::Paths;
    ///
    /// let paths = Paths::binary_paths();
    /// assert_eq!(paths, vec![
    ///     "/usr/local/bin/workflow".to_string(),
    /// ]);
    /// ```
    pub fn binary_paths() -> Vec<String> {
        let install_dir = Self::binary_install_dir();
        Self::command_names()
            .iter()
            .map(|name| format!("{}/{}", install_dir, name))
            .collect()
    }

    /// 获取 completion 目录的完整路径
    ///
    /// 返回 `~/.workflow/completions` 目录的完整路径。
    ///
    /// # 返回
    ///
    /// 返回 completion 目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::settings::paths::Paths;
    ///
    /// let dir = Paths::completion_dir()?;
    /// assert_eq!(dir, PathBuf::from("~/.workflow/completions"));
    /// ```
    pub fn completion_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_dir = PathBuf::from(home);
        Ok(home_dir.join(".workflow/completions"))
    }

    // ==================== Shell 路径相关方法 ====================
    /// 获取 shell 配置文件路径
    ///
    /// 支持的 shell 类型及其配置文件路径：
    /// - zsh → `~/.zshrc`
    /// - bash → `~/.bash_profile`（如果不存在则使用 `~/.bashrc`）
    /// - fish → `~/.config/fish/config.fish`
    /// - powershell → `~/.config/powershell/Microsoft.PowerShell_profile.ps1`
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
    ///
    /// # 示例
    ///
    /// ```
    /// use clap_complete::shells::Shell;
    /// use workflow::settings::paths::Paths;
    ///
    /// let zsh_path = Paths::config_file(&Shell::Zsh)?;
    /// assert_eq!(zsh_path, PathBuf::from("~/.zshrc"));
    /// ```
    pub fn config_file(shell: &Shell) -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_dir = PathBuf::from(home);

        let config_file = match shell {
            Shell::Zsh => home_dir.join(".zshrc"),
            Shell::Bash => {
                // macOS 通常使用 .bash_profile，Linux 使用 .bashrc
                // 这里优先使用 .bash_profile，如果不存在则使用 .bashrc
                let bash_profile = home_dir.join(".bash_profile");
                let bashrc = home_dir.join(".bashrc");

                // 如果 .bash_profile 不存在但 .bashrc 存在，使用 .bashrc
                // 否则使用 .bash_profile
                if !bash_profile.exists() && bashrc.exists() {
                    bashrc
                } else {
                    bash_profile
                }
            }
            Shell::Fish => home_dir.join(".config/fish/config.fish"),
            Shell::PowerShell => {
                home_dir.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
            }
            Shell::Elvish => home_dir.join(".elvish/rc.elv"),
            _ => anyhow::bail!("Unsupported shell type"),
        };

        Ok(config_file)
    }
}
