//! 安装命令
//! 提供安装二进制文件和 shell completion 的功能

use crate::{
    base::settings::paths::Paths, log_break, log_debug, log_info, log_success, log_warning,
    Completion, Detect,
};
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
    /// 自动检测 shell 类型（zsh/bash）并安装相应的 completion 脚本。
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
        // 配置文件 ~/.workflow/.completions 是统一配置，同时支持 zsh 和 bash
        // 配置文件会在运行时检测当前 shell 类型（通过 $ZSH_VERSION 和 $BASH_VERSION）
        // 因此需要同时生成 zsh 和 bash 的补全脚本文件，确保：
        // 1. 用户在不同 shell 环境下都能使用补全功能
        // 2. 用户切换 shell 时补全功能仍然可用
        // 3. 配置文件尝试加载补全脚本时文件存在
        log_debug!("正在生成 completion 脚本...");

        // 生成当前检测到的 shell 类型的补全脚本（确保当前环境可用）
        let shell_type_str = shell.to_string();
        log_debug!("生成 {} completion 脚本...", shell_type_str);
        Completion::generate_all_completions(
            Some(shell_type_str.clone()),
            Some(completion_dir.to_string_lossy().to_string()),
        )?;

        // 生成 zsh 补全脚本（配置文件支持 zsh，需要文件存在）
        if shell != Shell::Zsh {
            log_debug!("生成 zsh completion 脚本（配置文件需要）...");
            Completion::generate_all_completions(
                Some("zsh".to_string()),
                Some(completion_dir.to_string_lossy().to_string()),
            )?;
        }

        // 生成 bash 补全脚本（配置文件支持 bash，需要文件存在）
        if shell != Shell::Bash {
            log_debug!("生成 bash completion 脚本（配置文件需要）...");
            Completion::generate_all_completions(
                Some("bash".to_string()),
                Some(completion_dir.to_string_lossy().to_string()),
            )?;
        }

        // 配置 shell 配置文件
        log_debug!("正在配置 shell 配置文件...");
        Completion::configure_shell_config(&shell)?;

        log_success!("  shell completion 安装完成");
        log_break!();
        log_info!("提示：请运行以下命令重新加载配置:");
        log_info!("  source ~/.zshrc  # 或 source ~/.bashrc");

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
