//! 卸载命令实现
//!
//! 删除 Workflow CLI 的所有配置和二进制文件。

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::process::Command;

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use prompt::{br, info, print, success, warning, ConfirmBuilder};
use toolkit::{detect_shell, Paths, Reload};

use crate::registry::get_completion_service;

/// 卸载命令
pub struct UninstallCommand {
    /// 是否跳过确认
    force: bool,
    /// 是否保留配置文件
    keep_config: bool,
}

impl UninstallCommand {
    /// 创建新的 UninstallCommand 实例
    pub fn new(force: bool, keep_config: bool) -> Self {
        Self { force, keep_config }
    }

    /// 运行卸载流程
    pub fn run(&self) -> Result<()> {
        warning!("Uninstall Workflow CLI");
        br!();
        print!("This will remove all Workflow CLI configuration and binaries.");
        print!("This includes:");
        print!("  - Binary files: workflow");
        print!("  - Shell completion scripts");
        if !self.keep_config {
            print!("  - TOML configuration files (workflow.toml)");
        }
        br!();

        // 显示将要删除的二进制文件
        let existing_binaries = self.find_existing_binaries();

        if !existing_binaries.is_empty() {
            print!("Binary files to be removed:");
            for binary_path in &existing_binaries {
                print!("  - {}", binary_path);
            }
            br!();
        }

        // 确认删除
        if !self.force {
            let confirmed =
                ConfirmBuilder::new("Remove binary files and shell completion scripts?")
                    .default(false)
                    .prompt()
                    .wrap_err("Failed to get confirmation")?;

            if !confirmed {
                print!("Uninstall cancelled.");
                return Ok(());
            }
        }

        // 是否删除配置文件
        let remove_config = if self.keep_config {
            false
        } else if self.force {
            true
        } else {
            ConfirmBuilder::new("Remove TOML config file (workflow.toml)?")
                .default(true)
                .prompt()
                .wrap_err("Failed to get confirmation")?
        };

        // 删除二进制文件
        if !existing_binaries.is_empty() {
            br!();
            print!("Removing binary files...");
            self.remove_binaries(&existing_binaries)?;
        }

        // 删除 shell completion
        br!();
        print!("Removing shell completion scripts...");
        self.remove_completions()?;

        // 删除配置文件
        if remove_config {
            br!();
            print!("Removing configuration...");
            let removed_files = self.remove_config_files()?;
            if !removed_files.is_empty() {
                success!("Configuration removed successfully");
                for file in &removed_files {
                    print!("  - {} removed", file);
                }
            }
        } else {
            br!();
            print!("Configuration will be kept (not removed).");
        }

        br!();
        success!("Uninstall completed successfully!");

        if remove_config {
            print!("All Workflow CLI configuration has been removed.");
        } else {
            print!("Workflow CLI configuration has been kept (not removed).");
        }

        if !existing_binaries.is_empty() {
            print!("All Workflow CLI binary files have been removed.");
        }

        print!("All Workflow CLI shell completion scripts have been removed.");

        // 尝试重新加载 shell 配置
        br!();
        print!("Reloading shell configuration...");
        self.reload_shell_config();

        Ok(())
    }

    /// 查找存在的二进制文件
    fn find_existing_binaries(&self) -> Vec<String> {
        let binary_paths = Paths::binary_paths();
        let mut existing = Vec::new();

        for binary_path in binary_paths {
            let path = Path::new(&binary_path);
            if path.exists() {
                existing.push(binary_path);
            }
        }

        // 检查 install 二进制
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        let install_name = Paths::binary_name("install");
        let install_binary = install_path.join(install_name);
        if install_binary.exists() {
            existing.push(install_binary.to_string_lossy().to_string());
        }

        existing
    }

    /// 删除二进制文件
    fn remove_binaries(&self, _binaries: &[String]) -> Result<()> {
        let (removed, need_sudo) = self.try_remove_binaries()?;

        // 显示已删除的文件
        for binary_path in &removed {
            print!("  Removed: {}", binary_path);
        }

        // 处理需要 sudo 的文件
        if !need_sudo.is_empty() {
            #[cfg(unix)]
            {
                toolkit::log_debug!("Some files require sudo privileges, using sudo to remove...");
                for binary_path in &need_sudo {
                    match Command::new("sudo").arg("rm").arg("-f").arg(binary_path).status() {
                        Ok(status) if status.success() => {
                            print!("  Removed: {}", binary_path);
                        }
                        Ok(_) | Err(_) => {
                            warning!("Failed to remove {} with sudo", binary_path);
                            print!(
                                "  You may need to manually remove it with: sudo rm {}",
                                binary_path
                            );
                        }
                    }
                }
            }

            #[cfg(windows)]
            {
                warning!("Some files require administrator privileges.");
                print!("Please run this command as administrator or manually remove:");
                for binary_path in &need_sudo {
                    print!("  {}", binary_path);
                }
            }
        }

        // 删除 install 二进制（如果存在）
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        let install_name = Paths::binary_name("install");
        let install_binary = install_path.join(install_name);

        if install_binary.exists() {
            let install_binary_str = install_binary.to_string_lossy();

            #[cfg(unix)]
            {
                match Command::new("sudo")
                    .arg("rm")
                    .arg("-f")
                    .arg(install_binary_str.as_ref())
                    .status()
                {
                    Ok(status) if status.success() => {
                        print!("  Removed: {}", install_binary_str);
                    }
                    Ok(_) | Err(_) => {
                        warning!("Failed to remove {} with sudo", install_binary_str);
                        print!(
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
                        print!("  Removed: {}", install_binary_str);
                    }
                    Err(e) => {
                        warning!("Failed to remove {}: {}", install_binary_str, e);
                        print!(
                            "  You may need to manually remove it: {}",
                            install_binary_str
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// 尝试删除二进制文件
    ///
    /// 返回 (已删除列表, 需要sudo列表)
    fn try_remove_binaries(&self) -> Result<(Vec<String>, Vec<String>)> {
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

    /// 删除 shell completion
    fn remove_completions(&self) -> Result<()> {
        // 使用 completion service 删除
        let service = get_completion_service();
        let result = service
            .remove(true) // remove_all = true
            .wrap_err("Failed to remove completions")?;

        // 显示已删除的配置
        for shell in &result.removed_configs {
            info!("  Removed {} config", shell);
        }

        // 显示已删除的文件
        for file in &result.removed_files {
            info!("  Removed: {}", file.display());
        }

        // 显示已删除的配置文件
        if let Some(config_file) = &result.removed_config_file {
            info!("  Removed: {}", config_file.display());
        }

        // 显示失败的操作
        for (item, error) in &result.failures {
            warning!("Failed to remove {}: {}", item, error);
        }

        // 删除 completions 文件夹
        if let Ok(completion_dir) = Paths::completion_dir() {
            if completion_dir.exists() {
                // 先尝试删除空文件夹
                match fs::remove_dir(&completion_dir) {
                    Ok(_) => {
                        info!("  Removed: {}", completion_dir.display());
                    }
                    Err(_) => {
                        // 如果文件夹非空，删除整个文件夹
                        if let Err(e2) = fs::remove_dir_all(&completion_dir) {
                            toolkit::log_debug!(
                                "Could not remove completions directory: {} ({})",
                                completion_dir.display(),
                                e2
                            );
                        } else {
                            info!("  Removed: {}", completion_dir.display());
                        }
                    }
                }
            }
        }

        if !result.removed_files.is_empty() || result.removed_config_file.is_some() {
            success!("Completion scripts removed");
        }

        Ok(())
    }

    /// 删除配置文件
    fn remove_config_files(&self) -> Result<Vec<String>> {
        let mut removed = Vec::new();

        // 删除 workflow.toml
        if let Ok(workflow_config_path) = Paths::workflow_config() {
            if workflow_config_path.exists() {
                fs::remove_file(&workflow_config_path)
                    .wrap_err("Failed to remove workflow.toml")?;
                removed.push("workflow.toml".to_string());
            }
        }

        // 删除 jira.toml
        if let Ok(jira_config_path) = Paths::jira_config() {
            if jira_config_path.exists() {
                fs::remove_file(&jira_config_path).wrap_err("Failed to remove jira.toml")?;
                removed.push("jira.toml".to_string());
            }
        }

        // 删除 llm.toml
        if let Ok(llm_config_path) = Paths::llm_config() {
            if llm_config_path.exists() {
                fs::remove_file(&llm_config_path).wrap_err("Failed to remove llm.toml")?;
                removed.push("llm.toml".to_string());
            }
        }

        Ok(removed)
    }

    /// 重新加载 shell 配置
    fn reload_shell_config(&self) {
        if let Ok(shell) = detect_shell() {
            match Reload::shell(&shell) {
                Ok(result) if result.reloaded => {
                    info!("Shell configuration reloaded");
                }
                _ => {
                    self.print_reload_hint();
                }
            }
        } else {
            self.print_reload_hint();
        }
    }

    /// 打印重新加载提示
    fn print_reload_hint(&self) {
        print!("Could not automatically reload shell configuration.");
        print!("Please manually reload your shell configuration:");
        #[cfg(unix)]
        {
            print!("  source ~/.zshrc  # for zsh");
            print!("  source ~/.bashrc  # for bash");
        }
        #[cfg(windows)]
        {
            print!("  . $PROFILE  # for PowerShell");
        }
    }
}
