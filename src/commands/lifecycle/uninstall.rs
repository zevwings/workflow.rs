//! 卸载命令
//! 删除 Workflow CLI 的所有配置

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use duct::cmd;

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};

use crate::base::dialog::ConfirmDialog;
use crate::base::settings::paths::Paths;
use crate::base::shell::{Detect, Reload};
use crate::base::system::Clipboard;
use crate::{
    log_break, log_debug, log_info, log_message, log_success, log_warning, Completion, ProxyManager,
};

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
        log_message!("  - Binary files: workflow, install");
        log_message!("  - Shell completion scripts");
        log_break!();

        // 显示将要删除的二进制文件
        let binary_paths = Paths::binary_paths();
        let mut existing_binaries = Vec::new();
        for binary_path in &binary_paths {
            let path = Path::new(binary_path);
            if path.exists() {
                existing_binaries.push(binary_path.clone());
            }
        }

        // 检查 install 二进制
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        let install_name = Paths::binary_name("install");
        let install_binary = install_path.join(install_name);
        if install_binary.exists() {
            existing_binaries.push(install_binary.to_string_lossy().to_string());
        }

        if !existing_binaries.is_empty() {
            log_message!("Binary files to be removed:");
            for binary_path in &existing_binaries {
                log_message!("  - {}", binary_path);
            }
            log_break!();
        }

        // 第一步确认：是否删除二进制文件和 completion 脚本
        if !ConfirmDialog::new("Remove binary files and shell completion scripts?")
            .with_default(false)
            .prompt()?
        {
            log_message!("Uninstall cancelled.");
            return Ok(());
        }

        // 第二步确认：是否删除 TOML 配置文件
        let remove_config = ConfirmDialog::new("Remove TOML config file (workflow.toml)?")
            .with_default(true)
            .prompt()?;

        // 删除二进制文件
        if !existing_binaries.is_empty() {
            log_break!();
            log_message!("Removing binary files...");
            match Self::remove_binaries() {
                Ok((removed, need_sudo)) => {
                    if !removed.is_empty() {
                        for binary_path in &removed {
                            log_message!("  Removed: {}", binary_path);
                        }
                    }
                    if !need_sudo.is_empty() {
                        // 自动使用 sudo 删除需要权限的文件（仅 Unix）
                        #[cfg(unix)]
                        {
                            log_debug!(
                                "  Some files require sudo privileges, using sudo to remove..."
                            );
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
                        #[cfg(windows)]
                        {
                            log_warning!("  Some files require administrator privileges.");
                            log_message!(
                                "  Please run this command as administrator or manually remove:"
                            );
                            for binary_path in &need_sudo {
                                log_message!("    {}", binary_path);
                            }
                        }
                    }
                }
                Err(e) => {
                    log_warning!("  Failed to remove binary files: {}", e);
                    // 尝试使用 sudo 删除所有剩余的文件（仅 Unix）
                    #[cfg(unix)]
                    {
                        log_message!("Attempting to remove remaining files with sudo...");
                        for binary_path in &existing_binaries {
                            let path = Path::new(binary_path);
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
                    #[cfg(windows)]
                    {
                        log_warning!("  Some files could not be removed.");
                        log_message!(
                            "  Please run this command as administrator or manually remove:"
                        );
                        for binary_path in &existing_binaries {
                            let path = Path::new(binary_path);
                            if path.exists() {
                                log_message!("    {}", binary_path);
                            }
                        }
                    }
                }
            }

            // 删除 install 二进制（如果存在）
            if install_binary.exists() {
                let install_binary_str = install_binary.to_string_lossy();
                #[cfg(unix)]
                {
                    match cmd("sudo", &["rm", "-f", install_binary_str.as_ref()]).run() {
                        Ok(_) => {
                            log_message!("  Removed: {}", install_binary_str);
                        }
                        Err(e) => {
                            log_warning!(
                                "  Failed to remove {} with sudo: {}",
                                install_binary_str,
                                e
                            );
                            log_message!(
                                "  You may need to manually remove it with: sudo rm {}",
                                install_binary_str
                            );
                        }
                    }
                }
                #[cfg(windows)]
                {
                    match fs::remove_file(&install_binary) {
                        Ok(_) => {
                            log_message!("  Removed: {}", install_binary_str);
                        }
                        Err(e) => {
                            log_warning!("  Failed to remove {}: {}", install_binary_str, e);
                            log_message!(
                                "  You may need to manually remove it: {}",
                                install_binary_str
                            );
                        }
                    }
                }
            }
        }

        // 卸载 shell completion（只要第一步确认就删除）
        log_break!();
        log_message!("Removing shell completion scripts...");
        // 删除所有 shell 类型的 completion 文件（不依赖当前 shell）
        let removal_result =
            Completion::remove_completion_files(&clap_complete::shells::Shell::Zsh)?;

        // 显示删除的文件
        for file in &removal_result.removed_files {
            log_info!("  Removed: {}", file.display());
        }

        // 显示失败的文件
        for (file, error) in &removal_result.failed_files {
            log_info!("Failed to delete: {} ({})", file.display(), error);
        }

        if removal_result.removed_count > 0 {
            log_info!("  Completion script files removed");
        }

        // 删除 completions 文件夹
        let completion_dir = Paths::completion_dir();
        if let Ok(dir) = completion_dir {
            if dir.exists() {
                // 先尝试删除空文件夹，如果失败（非空）则删除整个文件夹及其内容
                match fs::remove_dir(&dir) {
                    Ok(_) => {
                        log_info!("  Removed: {}", dir.display());
                    }
                    Err(e) => {
                        // 如果文件夹非空，使用 remove_dir_all 删除整个文件夹
                        if e.kind() == std::io::ErrorKind::DirectoryNotEmpty {
                            match fs::remove_dir_all(&dir) {
                                Ok(_) => {
                                    log_info!("  Removed: {}", dir.display());
                                }
                                Err(e2) => {
                                    log_debug!(
                                        "  Could not remove completions directory: {} ({})",
                                        dir.display(),
                                        e2
                                    );
                                }
                            }
                        } else {
                            log_debug!(
                                "  Could not remove completions directory: {} ({})",
                                dir.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        let config_file_removed = Completion::remove_completion_config_file()?;
        if config_file_removed {
            log_info!(
                "  Removed: {}",
                Paths::local_base_dir()?.join(".completions").display()
            );
        } else {
            log_info!(
                "  Completion config file not found: {}",
                Paths::local_base_dir()?.join(".completions").display()
            );
        }
        // 移除所有 shell 的 completion 配置
        Completion::remove_all_completion_configs()?;

        // 删除配置（需要第二步确认）
        if remove_config {
            log_break!();
            log_message!("Removing configuration...");
            let removed_files =
                Self::remove_config_files().wrap_err("Failed to uninstall configuration")?;
            log_message!("Configuration removed successfully");
            for file in &removed_files {
                log_message!("  - {} removed", file);
            }
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
        if let Ok(shell) = Detect::shell() {
            let _ = Reload::shell(&shell);
        } else {
            log_break!();
            log_message!("Could not detect shell type.");
            log_message!("Please manually reload your shell configuration:");
            #[cfg(unix)]
            {
                log_message!("  source ~/.zshrc  # for zsh");
                log_message!("  source ~/.bashrc  # for bash");
            }
            #[cfg(windows)]
            {
                log_message!("  . $PROFILE  # for PowerShell");
            }
        }

        Ok(())
    }

    /// 从 shell 环境变量中移除代理设置
    fn remove_proxy_settings() -> Result<()> {
        let result = ProxyManager::disable().wrap_err("Failed to remove proxy settings")?;

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

    /// 删除所有 Workflow CLI 二进制文件
    ///
    /// 这会删除系统目录（通常是 /usr/local/bin）下的二进制文件。
    ///
    /// # 返回
    ///
    /// 返回一个元组，包含：
    /// - 成功删除的文件列表
    /// - 需要 sudo 权限的文件列表（权限不足）
    ///
    /// # 错误
    ///
    /// 如果删除文件时出现非权限错误，返回相应的错误信息。
    fn remove_binaries() -> Result<(Vec<String>, Vec<String>)> {
        let binary_paths = Paths::binary_paths();
        let mut removed = Vec::new();
        let mut need_sudo = Vec::new();

        for binary_path in binary_paths {
            let path = Path::new(&binary_path);
            if path.exists() {
                match fs::remove_file(path) {
                    Ok(_) => {
                        removed.push(binary_path);
                    }
                    Err(e) => {
                        // 检查是否是权限错误
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            need_sudo.push(binary_path);
                        } else {
                            return Err(eyre!(
                                "Failed to remove binary file: {}: {}",
                                binary_path,
                                e
                            ));
                        }
                    }
                }
            }
        }

        Ok((removed, need_sudo))
    }

    /// 删除所有 Workflow CLI TOML 配置文件
    ///
    /// 这会删除 workflow.toml 和 jira-users.toml。
    ///
    /// # 返回
    ///
    /// 返回成功删除的文件列表。
    ///
    /// # 错误
    ///
    /// 如果删除文件时出错，返回相应的错误信息。
    fn remove_config_files() -> Result<Vec<String>> {
        let mut removed = Vec::new();

        // 删除 workflow.toml
        if let Ok(workflow_config_path) = Paths::workflow_config() {
            if workflow_config_path.exists() {
                fs::remove_file(&workflow_config_path)
                    .wrap_err("Failed to remove workflow.toml")?;
                removed.push(crate::base::settings::paths::WORKFLOW_CONFIG_FILE.to_string());
            }
        }

        // 删除 jira-users.toml
        if let Ok(config_dir) = Paths::config_dir() {
            let jira_users_config_path = config_dir.join("jira-users.toml");
            if jira_users_config_path.exists() {
                fs::remove_file(&jira_users_config_path)
                    .wrap_err("Failed to remove jira-users.toml")?;
                removed.push("jira-users.toml".to_string());
            }
        }

        Ok(removed)
    }
}
