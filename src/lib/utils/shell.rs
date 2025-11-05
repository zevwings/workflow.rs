//! Shell 检测与管理工具
//! 提供 Shell 类型检测、配置路径管理等功能

use anyhow::{Context, Result};
use duct::cmd;
use std::path::PathBuf;

/// Shell 信息结构体
pub struct ShellInfo {
    pub shell_type: String,
    pub completion_dir: PathBuf,
    pub config_file: PathBuf,
}

/// Shell 管理工具
pub struct Shell;

impl Shell {
    /// 检测当前 shell 类型并返回 ShellInfo
    pub fn detect() -> Result<ShellInfo> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell_type = if shell.contains("zsh") {
            "zsh"
        } else if shell.contains("bash") {
            "bash"
        } else {
            anyhow::bail!("不支持的 shell: {}", shell);
        };

        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_dir = PathBuf::from(home);

        let (completion_dir, config_file) = if shell_type == "zsh" {
            (home_dir.join(".zsh/completions"), home_dir.join(".zshrc"))
        } else {
            (
                home_dir.join(".bash_completion.d"),
                home_dir.join(".bashrc"),
            )
        };

        Ok(ShellInfo {
            shell_type: shell_type.to_string(),
            completion_dir,
            config_file,
        })
    }

    /// 获取 shell 配置文件路径
    pub fn get_config_path(shell_type: &str) -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_dir = PathBuf::from(home);

        let config_file = if shell_type == "zsh" {
            home_dir.join(".zshrc")
        } else {
            home_dir.join(".bashrc")
        };

        Ok(config_file)
    }

    /// 获取 completion 目录路径
    pub fn get_completion_dir(shell_type: &str) -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_dir = PathBuf::from(home);

        let completion_dir = if shell_type == "zsh" {
            home_dir.join(".zsh/completions")
        } else {
            home_dir.join(".bash_completion.d")
        };

        Ok(completion_dir)
    }

    /// 重新加载 shell 配置（在子进程中执行 source 命令）
    pub fn reload_config(shell_info: &ShellInfo) -> Result<()> {
        use crate::{log_info, log_success, log_warning};

        let config_file = shell_info.config_file.display().to_string();
        let shell_cmd = format!("source {}", config_file);

        // 尝试在子 shell 中执行 source 命令
        // 注意：这不会影响当前 shell，但可以验证配置文件是否有效
        let status = cmd(&shell_info.shell_type, &["-c", &shell_cmd])
            .run()
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("Failed to reload config: {}", e));

        match status {
            Ok(_) => {
                log_success!("✓ Shell configuration reloaded (in subprocess)");
                log_info!("Note: Changes may not take effect in the current shell.");
                log_info!("Please run manually: source {}", config_file);
                Ok(())
            }
            Err(e) => {
                log_warning!("⚠️  Could not reload shell configuration: {}", e);
                log_info!("Please run manually: source {}", config_file);
                Err(e)
            }
        }
    }
}
