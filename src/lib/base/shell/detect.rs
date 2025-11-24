use anyhow::Result;
use clap_complete::shells::Shell;
use std::fs;

/// Shell 检测工具
///
/// 提供 Shell 类型检测功能。
pub struct Detect;

impl Detect {
    /// 检测当前 shell 类型并返回 Shell
    ///
    /// 根据 `SHELL` 环境变量检测当前 shell 类型。
    /// 支持的 shell 类型：zsh、bash、fish、powershell、elvish。
    ///
    /// # 返回
    ///
    /// 返回检测到的 `Shell` 类型。
    ///
    /// # 错误
    ///
    /// 如果 shell 类型不支持，返回相应的错误信息。
    pub fn shell() -> Result<Shell> {
        Shell::from_env()
            .or_else(|| {
                // 如果 from_env() 失败，尝试从 SHELL 环境变量解析
                std::env::var("SHELL").ok().and_then(Shell::from_shell_path)
            })
            .ok_or_else(|| {
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
                anyhow::anyhow!("Unsupported shell: {}", shell)
            })
    }

    /// 检测系统中已安装的 shell
    ///
    /// 通过读取 `/etc/shells` 文件来检测已安装的 shell。
    /// 如果无法读取文件或没有检测到任何 shell，至少返回当前 shell（如果可用）。
    ///
    /// # 返回
    ///
    /// 返回已安装的 shell 列表。
    pub fn installed_shells() -> Vec<Shell> {
        let mut installed = Vec::new();

        // 读取 /etc/shells 文件
        if let Ok(content) = fs::read_to_string("/etc/shells") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // 尝试从路径解析 shell 类型
                if let Some(shell) = Shell::from_shell_path(line) {
                    installed.push(shell);
                }
            }
        }

        // 如果没有从 /etc/shells 检测到，至少添加当前 shell
        if installed.is_empty() {
            if let Ok(current_shell) = Self::shell() {
                installed.push(current_shell);
            }
        }

        installed
    }
}
