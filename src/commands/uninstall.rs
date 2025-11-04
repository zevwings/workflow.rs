//! 卸载命令
//! 删除 Workflow CLI 的所有配置

use crate::{log_info, log_success, log_warning, EnvFile, Uninstall};
use anyhow::{Context, Result};
use dialoguer::Confirm;
use duct::cmd;

/// 卸载命令
pub struct UninstallCommand;

impl UninstallCommand {
    /// 运行卸载流程
    pub fn run() -> Result<()> {
        log_warning!("⚠️  Uninstall Workflow CLI\n");
        log_info!("This will remove all Workflow CLI configuration and binaries.");
        log_info!("This includes:");
        log_info!("  - All environment variables (EMAIL, JIRA_API_TOKEN, etc.)");
        log_info!("  - The entire Workflow CLI configuration block");
        log_info!("  - Binary files: workflow, pr, qk\n");

        let shell_config_path = EnvFile::get_shell_config_path()
            .map_err(|_| anyhow::anyhow!("Failed to get shell config path"))?;
        log_info!("Shell config: {:?}\n", shell_config_path);

        // 显示将要删除的二进制文件
        let binary_paths = Uninstall::get_binary_paths();
        let mut existing_binaries = Vec::new();
        for binary_path in &binary_paths {
            let path = std::path::Path::new(binary_path);
            if path.exists() {
                existing_binaries.push(*binary_path);
            }
        }

        if !existing_binaries.is_empty() {
            log_info!("Binary files to be removed:");
            for binary_path in &existing_binaries {
                log_info!("  - {}", binary_path);
            }
            log_info!("");
        }

        // 确认卸载
        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to uninstall?")
            .default(false)
            .interact()
            .context("Failed to get confirmation")?;

        if !confirmed {
            log_info!("Uninstall cancelled.");
            return Ok(());
        }

        // 确认是否删除配置
        let remove_config = Confirm::new()
            .with_prompt("Remove configuration?")
            .default(false)
            .interact()
            .context("Failed to get confirmation for removing configuration")?;

        // 执行卸载
        if remove_config {
            log_info!("\n🗑️  Removing configuration...");
            Uninstall::uninstall_all().context("Failed to uninstall configuration")?;
            log_info!("  ✓ Configuration removed successfully");
        } else {
            log_info!("\nℹ  Configuration will be kept (not removed).");
        }

        if !existing_binaries.is_empty() {
            log_info!("Removing binary files...");
            match Uninstall::remove_binaries() {
                Ok((removed, need_sudo)) => {
                    if !removed.is_empty() {
                        for binary_path in &removed {
                            log_info!("  ✓ Removed: {}", binary_path);
                        }
                    }
                    if !need_sudo.is_empty() {
                        // 自动使用 sudo 删除需要权限的文件
                        log_info!("  Some files require sudo privileges, using sudo to remove...");
                        for binary_path in &need_sudo {
                            match cmd("sudo", &["rm", binary_path]).run() {
                                Ok(_) => {
                                    log_info!("  ✓ Removed: {}", binary_path);
                                }
                                Err(e) => {
                                    log_warning!("  ⚠️  Failed to remove {} with sudo: {}", binary_path, e);
                                    log_info!("     You may need to manually remove it with: sudo rm {}", binary_path);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log_warning!("⚠️  Failed to remove binary files: {}", e);
                    // 尝试使用 sudo 删除所有剩余的文件
                    log_info!("  Attempting to remove remaining files with sudo...");
                    for binary_path in &existing_binaries {
                        let path = std::path::Path::new(binary_path);
                        if path.exists() {
                            match cmd("sudo", &["rm", binary_path]).run() {
                                Ok(_) => {
                                    log_info!("  ✓ Removed: {}", binary_path);
                                }
                                Err(e) => {
                                    log_warning!("  ⚠️  Failed to remove {} with sudo: {}", binary_path, e);
                                    log_info!("     You may need to manually remove it with: sudo rm {}", binary_path);
                                }
                            }
                        }
                    }
                }
            }
        }

        log_success!("\n✅ Uninstall completed successfully!");
        if remove_config {
            log_info!("All Workflow CLI configuration has been removed from your shell config file.");
        } else {
            log_info!("Workflow CLI configuration has been kept (not removed).");
        }
        if !existing_binaries.is_empty() {
            log_info!("All Workflow CLI binary files have been removed.");
        }
        if remove_config {
            log_info!("Note: You may need to restart your shell or run 'source ~/.zshrc' (or similar) for changes to take effect.");
        }

        Ok(())
    }
}
