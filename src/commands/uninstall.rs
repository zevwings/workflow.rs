//! 卸载命令
//! 删除 Workflow CLI 的所有配置

use crate::{Completion, EnvFile, Shell, Uninstall, log_info, log_break, log_success, log_warning};
use anyhow::{Context, Result};
use dialoguer::Confirm;
use duct::cmd;

/// 卸载命令
pub struct UninstallCommand;

impl UninstallCommand {
    /// 运行卸载流程（一次性清理全部）
    pub fn run() -> Result<()> {
        log_warning!("  Uninstall Workflow CLI");
        log_break!();
        log_info!("This will remove all Workflow CLI configuration and binaries.");
        log_info!("This includes:");
        log_info!("  - All environment variables (EMAIL, JIRA_API_TOKEN, etc.)");
        log_info!("  - The entire Workflow CLI configuration block");
        log_info!("  - Binary files: workflow, pr, qk, install");
        log_info!("  - Shell completion scripts");
        log_break!();

        let shell_config_path = EnvFile::get_shell_config_path()
            .map_err(|_| anyhow::anyhow!("Failed to get shell config path"))?;
        log_info!("Shell config: {:?}", shell_config_path);
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
            log_info!("Binary files to be removed:");
            for binary_path in &existing_binaries {
                log_info!("  - {}", binary_path);
            }
            log_break!();
        }

        // 第一步确认：是否删除二进制文件和 completion 脚本
        let remove_binaries = Confirm::new()
            .with_prompt("Remove binary files and shell completion scripts?")
            .default(false)
            .interact()
            .context("Failed to get confirmation for removing binaries")?;

        if !remove_binaries {
            log_info!("Uninstall cancelled.");
            return Ok(());
        }

        // 第二步确认：是否删除环境变量配置
        let remove_config = Confirm::new()
            .with_prompt("Remove environment variables and configuration from shell config file?")
            .default(true)
            .interact()
            .context("Failed to get confirmation for removing configuration")?;

        // 删除二进制文件
        if !existing_binaries.is_empty() {
            log_break!();
            log_info!("  Removing binary files...");
            match Uninstall::remove_binaries() {
                Ok((removed, need_sudo)) => {
                    if !removed.is_empty() {
                        for binary_path in &removed {
                            log_info!("  Removed: {}", binary_path);
                        }
                    }
                    if !need_sudo.is_empty() {
                        // 自动使用 sudo 删除需要权限的文件
                        log_info!("  Some files require sudo privileges, using sudo to remove...");
                        for binary_path in &need_sudo {
                            match cmd("sudo", &["rm", "-f", binary_path]).run() {
                                Ok(_) => {
                                    log_info!("  Removed: {}", binary_path);
                                }
                                Err(e) => {
                                    log_warning!(
                                        "    Failed to remove {} with sudo: {}",
                                        binary_path,
                                        e
                                    );
                                    log_info!(
                                        "     You may need to manually remove it with: sudo rm {}",
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
                    log_info!("  Attempting to remove remaining files with sudo...");
                    for binary_path in &existing_binaries {
                        let path = std::path::Path::new(binary_path);
                        if path.exists() {
                            match cmd("sudo", &["rm", "-f", binary_path]).run() {
                                Ok(_) => {
                                    log_info!("  Removed: {}", binary_path);
                                }
                                Err(e) => {
                                    log_warning!(
                                        "    Failed to remove {} with sudo: {}",
                                        binary_path,
                                        e
                                    );
                                    log_info!(
                                        "     You may need to manually remove it with: sudo rm {}",
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
                        log_info!("  Removed: {}", install_path);
                    }
                    Err(e) => {
                        log_warning!("  Failed to remove {} with sudo: {}", install_path, e);
                        log_info!(
                            "     You may need to manually remove it with: sudo rm {}",
                            install_path
                        );
                    }
                }
            }
        }

        // 卸载 shell completion（只要第一步确认就删除）
        log_break!();
        log_info!("  Removing shell completion scripts...");
        if let Ok(shell_info) = Shell::detect() {
            Completion::remove_completion_files(&shell_info)?;
            Completion::remove_completion_config_file()?;
            if shell_info.config_file.exists() {
                Completion::remove_completion_config(&shell_info)?;
            } else {
                log_info!(
                    "  Config file {} does not exist",
                    shell_info.config_file.display()
                );
            }
        }

        // 删除配置（需要第二步确认）
        if remove_config {
            log_break!();
            log_info!("  Removing configuration...");
            Uninstall::uninstall_all().context("Failed to uninstall configuration")?;
            log_info!("  Configuration removed successfully");
        } else {
            log_break!();
            log_info!("  Configuration will be kept (not removed).");
        }

        log_break!();
        log_success!("  Uninstall completed successfully!");
        if remove_config {
            log_break!();
            log_info!(
                "All Workflow CLI configuration has been removed from your shell config file."
            );
        } else {
            log_info!("Workflow CLI configuration has been kept (not removed).");
        }
        if !existing_binaries.is_empty() {
            log_info!("All Workflow CLI binary files have been removed.");
        }
        log_info!("All Workflow CLI shell completion scripts have been removed.");

        // 尝试重新加载 shell 配置
        log_break!();
        log_info!("  Reloading shell configuration...");
        if let Ok(shell_info) = Shell::detect() {
            let _ = Shell::reload_config(&shell_info);
        } else {
            log_break!();
            log_info!("  Could not detect shell type.");
            log_info!("Please manually reload your shell configuration:");
            log_info!("  source ~/.zshrc  # for zsh");
            log_info!("  source ~/.bashrc  # for bash");
        }

        Ok(())
    }
}
