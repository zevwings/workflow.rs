//! 安装命令
//! 提供安装二进制文件和 shell completion 的功能

use crate::base::settings::paths::Paths;
use crate::base::shell::Detect;
use crate::{log_break, log_debug, log_info, log_success, log_warning, Completion};
use anyhow::{Context, Result};
use clap_complete::shells::Shell;
use std::fs;
use std::process::Command;

/// 安装命令
#[allow(dead_code)]
pub struct InstallCommand;

#[allow(dead_code)]
impl InstallCommand {
    /// 安装 shell completion 脚本（公共方法）
    ///
    /// 自动检测当前 shell 类型并安装相应的 completion 脚本。
    /// 只生成当前 shell 类型的 completion 脚本，简化安装流程。
    pub fn install_completions() -> Result<()> {
        log_info!("安装 shell completion 脚本...");

        let shell = Detect::shell()?;
        let completion_dir = Paths::completion_dir()?;

        log_debug!("检测 shell 类型...");
        log_debug!("检测到: {}", shell);

        // 创建 completion 目录
        fs::create_dir_all(&completion_dir).context("Failed to create completion directory")?;
        log_debug!("Completion 目录: {}", completion_dir.display());

        // 生成 completion 脚本
        // 只生成当前检测到的 shell 类型的补全脚本
        log_debug!("正在生成 completion 脚本...");

        let shell_type_str = shell.to_string();
        log_debug!("生成 {} completion 脚本...", shell_type_str);
        Completion::generate_all_completions(
            Some(shell_type_str),
            Some(completion_dir.to_string_lossy().to_string()),
        )?;

        // 配置 shell 配置文件
        log_debug!("正在配置 shell 配置文件...");
        Completion::configure_shell_config(&shell)?;

        log_success!("  shell completion 安装完成");
        log_break!();

        // 根据检测到的 shell 类型提示相应的重新加载命令
        let reload_hint = match shell {
            Shell::Zsh => "source ~/.zshrc",
            Shell::Bash => "source ~/.bash_profile  # 或 source ~/.bashrc",
            Shell::Fish => "重新打开终端或运行: source ~/.config/fish/config.fish",
            Shell::PowerShell => "重新打开 PowerShell 或运行: . $PROFILE",
            Shell::Elvish => "重新打开终端或运行: source ~/.elvish/rc.elv",
            _ => "请重新打开终端或重新加载 shell 配置文件",
        };
        log_info!("提示：请运行以下命令重新加载配置:");
        log_info!("  {}", reload_hint);

        Ok(())
    }

    /// 安装二进制文件到 /usr/local/bin
    ///
    /// 在当前可执行文件所在目录查找 workflow、pr、qk 二进制文件，
    /// 并将它们复制到 /usr/local/bin。
    pub fn install_binaries() -> Result<()> {
        log_info!("正在安装二进制文件到 /usr/local/bin...");

        // 获取当前可执行文件所在目录
        let current_exe =
            std::env::current_exe().context("Failed to get current executable path")?;
        let current_dir = current_exe
            .parent()
            .context("Failed to get parent directory of executable")?;

        log_debug!("当前目录: {}", current_dir.display());

        let binaries = ["workflow", "pr", "qk"];
        let mut installed_count = 0;

        for binary in &binaries {
            let source = current_dir.join(binary);
            let target = format!("/usr/local/bin/{}", binary);

            if !source.exists() {
                log_warning!("⚠  二进制文件 {} 不存在，跳过", source.display());
                continue;
            }

            log_info!("  安装 {} -> {}", binary, target);

            // 使用 sudo 复制文件
            let status = Command::new("sudo")
                .arg("cp")
                .arg(&source)
                .arg(&target)
                .status()
                .context(format!("Failed to copy {} to {}", source.display(), target))?;

            if !status.success() {
                anyhow::bail!("安装 {} 失败", binary);
            }

            // 设置执行权限
            Command::new("sudo")
                .arg("chmod")
                .arg("+x")
                .arg(&target)
                .status()
                .context(format!(
                    "Failed to set executable permission for {}",
                    target
                ))?;

            log_success!("    ✓  {} 安装完成", binary);
            installed_count += 1;
        }

        if installed_count > 0 {
            log_success!("  二进制文件安装完成（{} 个已安装）", installed_count);
            log_info!("已安装的命令:");
            log_info!("  - workflow (主命令)");
            log_info!("  - pr (PR 操作命令)");
            log_info!("  - qk (快速日志操作命令)");
        } else {
            anyhow::bail!("没有找到可安装的二进制文件");
        }

        Ok(())
    }
}
