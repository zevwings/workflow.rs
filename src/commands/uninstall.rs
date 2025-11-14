//! 卸载命令
//! 删除 Workflow CLI 的所有配置

use crate::{
    confirm, log_break, log_debug, log_message, log_success, log_warning, Clipboard, Completion,
    Proxy, Shell, Uninstall,
};
use anyhow::{Context, Result};
use duct::cmd;

/// 卸载命令
pub struct UninstallCommand;

impl UninstallCommand {
    /// 运行卸载流程（一次性清理全部）
    pub fn run() -> Result<()> {
        log_warning!("  Uninstall Workflow CLI");
        log_break!();
        log_message!("This will remove all Workflow CLI configuration and binaries.");
        log_message!("This includes:");
        log_message!("  - TOML configuration files (workflow.toml)");
        log_message!("  - Binary files: workflow, pr, qk, install");
        log_message!("  - Shell completion scripts");
        log_break!();

        // 显示将要删除的二进制文件
        let binary_paths = Uninstall::get_binary_paths();
        let mut existing_binaries = Vec::new();
        for binary_path in &binary_paths {
            let path = std::path::Path::new(binary_path);
            if path.exists() {
                existing_binaries.push(*binary_path);
            }
        }

        // 检查 install 二进制
        let install_path = "/usr/local/bin/install";
        if std::path::Path::new(install_path).exists() {
            existing_binaries.push(install_path);
        }

        if !existing_binaries.is_empty() {
            log_message!("Binary files to be removed:");
            for binary_path in &existing_binaries {
                log_message!("  - {}", binary_path);
            }
            log_break!();
        }

        // 第一步确认：是否删除二进制文件和 completion 脚本
        if !confirm(
            "Remove binary files and shell completion scripts?",
            false,
            None,
        )? {
            log_message!("Uninstall cancelled.");
            return Ok(());
        }

        // 第二步确认：是否删除 TOML 配置文件
        let remove_config = confirm("Remove TOML config file (workflow.toml)?", true, None)?;

        // 删除二进制文件
        if !existing_binaries.is_empty() {
            log_break!();
            log_message!("Removing binary files...");
            match Uninstall::remove_binaries() {
                Ok((removed, need_sudo)) => {
                    if !removed.is_empty() {
                        for binary_path in &removed {
                            log_message!("  Removed: {}", binary_path);
                        }
                    }
                    if !need_sudo.is_empty() {
                        // 自动使用 sudo 删除需要权限的文件
                        log_debug!("  Some files require sudo privileges, using sudo to remove...");
                        for binary_path in &need_sudo {
                            match cmd("sudo", &["rm", "-f", binary_path]).run() {
                                Ok(_) => {
                                    log_message!("  Removed: {}", binary_path);
                                }
                                Err(e) => {
                                    log_warning!(
                                        "    Failed to remove {} with sudo: {}",
                                        binary_path,
                                        e
                                    );
                                    log_message!(
                                        "  You may need to manually remove it with: sudo rm {}",
                                        binary_path
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log_warning!("  Failed to remove binary files: {}", e);
                    // 尝试使用 sudo 删除所有剩余的文件
                    log_message!("Attempting to remove remaining files with sudo...");
                    for binary_path in &existing_binaries {
                        let path = std::path::Path::new(binary_path);
                        if path.exists() {
                            match cmd("sudo", &["rm", "-f", binary_path]).run() {
                                Ok(_) => {
                                    log_message!("  Removed: {}", binary_path);
                                }
                                Err(e) => {
                                    log_warning!(
                                        "    Failed to remove {} with sudo: {}",
                                        binary_path,
                                        e
                                    );
                                    log_message!(
                                        "  You may need to manually remove it with: sudo rm {}",
                                        binary_path
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // 删除 install 二进制（如果存在）
            if std::path::Path::new(install_path).exists() {
                match cmd("sudo", &["rm", "-f", install_path]).run() {
                    Ok(_) => {
                        log_message!("  Removed: {}", install_path);
                    }
                    Err(e) => {
                        log_warning!("  Failed to remove {} with sudo: {}", install_path, e);
                        log_message!(
                            "  You may need to manually remove it with: sudo rm {}",
                            install_path
                        );
                    }
                }
            }
        }

        // 卸载 shell completion（只要第一步确认就删除）
        log_break!();
        log_message!("Removing shell completion scripts...");
        if let Ok(shell_info) = Shell::detect() {
            Completion::remove_completion_files(&shell_info)?;
            Completion::remove_completion_config_file()?;
            if shell_info.config_file.exists() {
                Completion::remove_completion_config(&shell_info)?;
            } else {
                log_message!(
                    "Config file {} does not exist",
                    shell_info.config_file.display()
                );
            }
        }

        // 删除配置（需要第二步确认）
        if remove_config {
            log_break!();
            log_message!("Removing configuration...");
            Uninstall::uninstall_all().context("Failed to uninstall configuration")?;
            log_message!("Configuration removed successfully");
            log_message!("  - workflow.toml removed");
        } else {
            log_break!();
            log_message!("Configuration will be kept (not removed).");
        }

        // 关闭代理（从 shell 环境变量中移除）
        log_break!();
        log_message!("Removing proxy settings from shell configuration...");
        Self::remove_proxy_settings()?;

        log_break!();
        log_success!("  Uninstall completed successfully!");
        if remove_config {
            log_break!();
            log_message!("All Workflow CLI configuration has been removed from TOML files.");
        } else {
            log_message!("Workflow CLI configuration has been kept (not removed).");
        }
        if !existing_binaries.is_empty() {
            log_message!("All Workflow CLI binary files have been removed.");
        }
        log_message!("All Workflow CLI shell completion scripts have been removed.");

        // 尝试重新加载 shell 配置
        log_break!();
        log_message!("Reloading shell configuration...");
        if let Ok(shell_info) = Shell::detect() {
            let _ = Shell::reload_config(&shell_info);
        } else {
            log_break!();
            log_message!("Could not detect shell type.");
            log_message!("Please manually reload your shell configuration:");
            log_message!("  source ~/.zshrc  # for zsh");
            log_message!("  source ~/.bashrc  # for bash");
        }

        Ok(())
    }

    /// 从 shell 环境变量中移除代理设置
    /// 使用 Proxy::disable_proxy() 公共方法
    fn remove_proxy_settings() -> Result<()> {
        let result = Proxy::disable_proxy().context("Failed to remove proxy settings")?;

        if !result.found_proxy {
            log_message!("No proxy settings found in shell configuration.");
            return Ok(());
        }

        if let Some(ref shell_config_path) = result.shell_config_path {
            log_success!("  Proxy settings removed from {:?}", shell_config_path);
        }

        if let Some(ref unset_cmd) = result.unset_command {
            log_message!("Proxy unset command: {}", unset_cmd);
            // 复制到剪贴板（静默处理，失败不影响卸载流程）
            let _ = Clipboard::copy(unset_cmd);
        }

        Ok(())
    }
}
