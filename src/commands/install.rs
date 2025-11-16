//! 安装命令
//! 提供安装 shell completion 的功能

use crate::{log_break, log_debug, log_info, log_success, log_warning, Completion, Shell};
use anyhow::{Context, Result};
use std::fs;

/// 安装命令
#[allow(dead_code)]
pub struct InstallCommand;

#[allow(dead_code)]
impl InstallCommand {

    /// 安装 shell completion 脚本（自动检测 shell，内部方法）
    fn install_completions() -> Result<()> {
        let shell_info = Shell::detect()?;

        log_debug!("检测 shell 类型...");
        log_debug!("检测到: {}", shell_info.shell_type);

        // 创建 completion 目录
        fs::create_dir_all(&shell_info.completion_dir)
            .context("Failed to create completion directory")?;
        log_debug!("Completion 目录: {}", shell_info.completion_dir.display());

        // 生成 completion 脚本（同时生成 zsh 和 bash 以支持统一配置）
        log_debug!("正在生成 completion 脚本...");
        log_debug!("生成 zsh completion 脚本...");
        Completion::generate_all_completions(
            Some("zsh".to_string()),
            Some(shell_info.completion_dir.to_string_lossy().to_string()),
        )?;
        log_debug!("生成 bash completion 脚本...");
        Completion::generate_all_completions(
            Some("bash".to_string()),
            Some(shell_info.completion_dir.to_string_lossy().to_string()),
        )?;

        // 配置 shell 配置文件
        log_debug!("正在配置 shell 配置文件...");
        Completion::configure_shell_config(&shell_info)?;

        log_success!("  shell completion 安装完成");
        log_info!("请运行以下命令重新加载配置:");
        log_info!("  source {}", shell_info.config_file.display());

        Ok(())
    }

    /// 安装 shell completion 脚本（用于 Makefile 调用）
    pub fn install() -> Result<()> {
        log_info!("安装 shell completion 脚本...");
        if let Err(e) = Self::install_completions() {
            log_warning!("⚠  shell completion 安装失败: {}", e);
            log_info!("  可稍后手动安装 completion 脚本");
            return Err(e);
        }
        log_success!("  shell completion 安装完成");
        log_break!();
        log_info!("提示：请运行以下命令重新加载配置:");
        log_info!("  source ~/.zshrc  # 或 source ~/.bashrc");
        Ok(())
    }
}
