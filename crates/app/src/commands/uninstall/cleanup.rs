//! 卸载命令实现
//!
//! 删除 Workflow CLI 的所有配置和二进制文件。
//! 路径统一从 pathService 获取；若路径不存在则需确认后再加入操作列表。

use std::{fs, path::PathBuf};

#[cfg(unix)]
use std::process::Command;

use color_eyre::{eyre::WrapErr, Result};
use prompt::{br, info, print, success, warning, ConfirmBuilder};
use toolkit::{detect_shell, reload_shell};

use crate::registry::{get_completion_service, get_path_service};

/// 卸载命令
pub struct UninstallCommand {
    /// 是否保留配置文件
    keep_config: bool,
}

impl UninstallCommand {
    /// 创建新的 UninstallCommand 实例
    pub fn new(keep_config: bool) -> Self {
        Self { keep_config }
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

        // 仅卸载 workflow 主二进制（从 pathService 获取路径，不存在则确认后再加入）
        let binary_to_remove = self.get_workflow_binary_path()?;
        if let Some(path) = binary_to_remove {
            self.remove_binary(path)?;

            // 是否删除配置文件
            let remove_config = if self.keep_config {
                false
            } else {
                ConfirmBuilder::new("Remove TOML config file (workflow.toml)?")
                    .default(true)
                    .prompt()
                    .wrap_err("Failed to get confirmation")?
            };

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

            print!("All Workflow CLI shell completion scripts have been removed.");

            // 尝试重新加载 shell 配置
            br!();
            print!("Reloading shell configuration...");
            self.reload_shell_config();
        } else {
            warning!("Workflow binary not found.");
        }
        Ok(())
    }

    /// 返回待删除的 workflow 主二进制路径（仅此一个）；若路径不存在则确认后再返回。
    fn get_workflow_binary_path(&self) -> Result<Option<PathBuf>> {
        let path_service = get_path_service();
        let install_dir = path_service
            .get_binary_install_dir()
            .wrap_err("Failed to get binary install dir")?;
        let bin_name = path_service.get_binary_name().wrap_err("Failed to get binary name")?;

        let workflow_binary = install_dir.join(&bin_name);

        let include = workflow_binary.exists()
            || self.should_include_missing_path(
                &workflow_binary.display().to_string(),
                "workflow binary",
            )?;
        Ok(include.then_some(workflow_binary))
    }

    /// 路径不存在时询问是否仍加入操作列表
    fn should_include_missing_path(&self, path: &str, kind: &str) -> Result<bool> {
        let confirmed = ConfirmBuilder::new(format!(
            "Path does not exist: {} ({}). Include it anyway?",
            path, kind
        ))
        .default(false)
        .prompt()
        .wrap_err("Failed to get confirmation")?;
        Ok(confirmed)
    }

    /// 删除二进制文件（列表来自 pathService 收集的路径）
    fn remove_binary(&self, binary_path: PathBuf) -> Result<()> {
        match self.try_remove_binary(binary_path.clone()) {
            Ok(()) => {
                success!("Workflow binary removed successfully");
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                #[cfg(unix)]
                {
                    toolkit::log_debug!(
                        "Some files require sudo privileges, using sudo to remove..."
                    );
                    match Command::new("sudo").arg("rm").arg("-f").arg(&binary_path).status() {
                        Ok(status) if status.success() => {
                            success!("Workflow binary removed successfully");
                        }
                        Ok(_) | Err(_) => {
                            warning!("Failed to remove {} with sudo", binary_path.display());
                            print!(
                                "  You may need to manually remove it with: sudo rm {}",
                                binary_path.display()
                            );
                        }
                    }
                }
                #[cfg(windows)]
                {
                    let _ = e;
                    warning!("Some files require administrator privileges.");
                    print!("Please run this command as administrator or manually remove:");
                    print!("  {}", binary_path.display());
                }
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    /// 尝试删除二进制文件（若路径存在则删除，否则静默成功）
    fn try_remove_binary(&self, bin_path: PathBuf) -> Result<(), std::io::Error> {
        if bin_path.exists() {
            match fs::remove_file(bin_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
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
        let path_service = get_path_service();
        if let Ok(comp_dir) = path_service.get_completion_dir() {
            if comp_dir.exists() {
                // 先尝试删除空文件夹
                match fs::remove_dir(&comp_dir) {
                    Ok(_) => {
                        info!("  Removed: {}", comp_dir.display());
                    }
                    Err(_) => {
                        // 如果文件夹非空，删除整个文件夹
                        if let Err(e2) = fs::remove_dir_all(&comp_dir) {
                            toolkit::log_debug!(
                                "Could not remove completions directory: {} ({})",
                                comp_dir.display(),
                                e2
                            );
                        } else {
                            info!("  Removed: {}", comp_dir.display());
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
        let mut removed: Vec<String> = Vec::new();
        let path_service = get_path_service();

        // 删除 workflow.toml
        if let Ok(workflow_config_filepath) = path_service.get_workflow_config_filepath() {
            // let wf_config_path = config_dir.join(WORKFLOW_CONFIG_FILE);
            if workflow_config_filepath.exists() {
                fs::remove_file(workflow_config_filepath)
                    .wrap_err("Failed to remove workflow.toml")?;
                removed.push("workflow.toml".to_string());
            }
        }

        // 删除 jira.toml
        if let Ok(jira_config_path) = path_service.get_jira_config_filepath() {
            if jira_config_path.exists() {
                fs::remove_file(&jira_config_path).wrap_err("Failed to remove jira.toml")?;
                removed.push("jira.toml".to_string());
            }
        }

        Ok(removed)
    }

    /// 重新加载 shell 配置
    fn reload_shell_config(&self) {
        if let Ok(shell) = detect_shell() {
            match reload_shell(&shell) {
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
