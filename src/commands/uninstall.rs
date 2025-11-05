//! 卸载命令
//! 删除 Workflow CLI 的所有配置

use crate::{log_info, log_success, log_warning, EnvFile, Uninstall};
use anyhow::{Context, Result};
use dialoguer::Confirm;
use duct::cmd;
use std::fs;
use std::path::PathBuf;

/// 卸载命令
pub struct UninstallCommand;

impl UninstallCommand {
    /// 运行卸载流程（一次性清理全部）
    pub fn run() -> Result<()> {
        log_warning!("⚠️  Uninstall Workflow CLI\n");
        log_info!("This will remove all Workflow CLI configuration and binaries.");
        log_info!("This includes:");
        log_info!("  - All environment variables (EMAIL, JIRA_API_TOKEN, etc.)");
        log_info!("  - The entire Workflow CLI configuration block");
        log_info!("  - Binary files: workflow, pr, qk, install");
        log_info!("  - Shell completion scripts\n");

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
            log_info!("");
        }

        // 确认卸载
        let confirmed = Confirm::new()
            .with_prompt("Are you sure you want to uninstall everything?")
            .default(false)
            .interact()
            .context("Failed to get confirmation")?;

        if !confirmed {
            log_info!("Uninstall cancelled.");
            return Ok(());
        }

        // 删除配置
        log_info!("\n🗑️  Removing configuration...");
        Uninstall::uninstall_all().context("Failed to uninstall configuration")?;
        log_info!("  ✓ Configuration removed successfully");

        // 删除二进制文件
        if !existing_binaries.is_empty() {
            log_info!("\n🗑️  Removing binary files...");
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
                            match cmd("sudo", &["rm", "-f", binary_path]).run() {
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
                            match cmd("sudo", &["rm", "-f", binary_path]).run() {
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

            // 删除 install 二进制（如果存在）
            if std::path::Path::new(install_path).exists() {
                match cmd("sudo", &["rm", "-f", install_path]).run() {
                    Ok(_) => {
                        log_info!("  ✓ Removed: {}", install_path);
                    }
                    Err(e) => {
                        log_warning!("  ⚠️  Failed to remove {} with sudo: {}", install_path, e);
                        log_info!("     You may need to manually remove it with: sudo rm {}", install_path);
                    }
                }
            }
        }

        // 卸载 shell completion
        log_info!("\n🗑️  Removing shell completion scripts...");
        Self::remove_completion_files_and_config()?;

        log_success!("\n✅ Uninstall completed successfully!");
        log_info!("All Workflow CLI configuration has been removed from your shell config file.");
        if !existing_binaries.is_empty() {
            log_info!("All Workflow CLI binary files have been removed.");
        }
        log_info!("All Workflow CLI shell completion scripts have been removed.");
        log_info!("Note: You may need to restart your shell or run 'source ~/.zshrc' (or similar) for changes to take effect.");

        Ok(())
    }

    /// 删除 shell completion 文件和配置（内部方法）
    fn remove_completion_files_and_config() -> Result<()> {
        let shell_info = Self::detect_shell()?;

        // 删除 completion 脚本文件
        let completion_files = if shell_info.shell_type == "zsh" {
            vec![
                shell_info.completion_dir.join("_workflow"),
                shell_info.completion_dir.join("_pr"),
                shell_info.completion_dir.join("_qk"),
            ]
        } else {
            vec![
                shell_info.completion_dir.join("workflow.bash"),
                shell_info.completion_dir.join("pr.bash"),
                shell_info.completion_dir.join("qk.bash"),
            ]
        };

        let mut removed_count = 0;
        for file in &completion_files {
            if file.exists() {
                if let Err(e) = fs::remove_file(file) {
                    log_warning!("⚠  删除失败: {} ({})", file.display(), e);
                } else {
                    log_info!("  ✓ Removed: {}", file.display());
                    removed_count += 1;
                }
            }
        }

        if removed_count > 0 {
            log_info!("  ✓ Completion script files removed");
        } else {
            log_info!("  ℹ  Completion script files not found (may not be installed)");
        }

        // 从配置文件中删除 completion 配置
        if shell_info.config_file.exists() {
            Self::remove_completion_config(&shell_info)?;
        } else {
            log_info!("  ℹ  Config file {} does not exist", shell_info.config_file.display());
        }

        Ok(())
    }

    /// 检测 shell 类型
    fn detect_shell() -> Result<ShellInfo> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell_type = if shell.contains("zsh") {
            "zsh"
        } else if shell.contains("bash") {
            "bash"
        } else {
            anyhow::bail!("不支持的 shell: {}", shell);
        };

        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_dir = PathBuf::from(home);

        let (completion_dir, config_file) = if shell_type == "zsh" {
            (home_dir.join(".zsh/completions"), home_dir.join(".zshrc"))
        } else {
            (
                home_dir.join(".bash_completion.d"),
                home_dir.join(".bashrc"),
            )
        };

        Ok(ShellInfo {
            shell_type: shell_type.to_string(),
            completion_dir,
            config_file,
        })
    }

    /// 从配置文件中删除 completion 配置
    fn remove_completion_config(shell_info: &ShellInfo) -> Result<()> {
        let config_content = fs::read_to_string(&shell_info.config_file)
            .unwrap_or_else(|_| String::new());

        let has_completion_block = config_content.contains("# Workflow CLI completions");
        let completion_dir_str = shell_info.completion_dir.display().to_string();
        let fpath_pattern = if shell_info.shell_type == "zsh" {
            format!("fpath=({} $fpath)", completion_dir_str)
        } else {
            String::new()
        };

        // 检查是否有 fpath 配置（仅在 zsh 中）
        let mut has_fpath = if shell_info.shell_type == "zsh" && !fpath_pattern.is_empty() {
            config_content.contains(&fpath_pattern)
        } else {
            false
        };

        if !has_completion_block && !has_fpath {
            log_info!("ℹ  completion 配置未在 {} 中找到", shell_info.config_file.display());
            return Ok(());
        }

        // 删除配置块
        let marker_start = "# Workflow CLI completions";
        let mut new_content = String::new();
        let lines: Vec<&str> = config_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // 检查是否是配置块开始
            if line.contains(marker_start) {
                // 跳过整个配置块
                if shell_info.shell_type == "zsh" {
                    // 跳过到 autoload 行之后
                    i += 1; // 跳过 marker 行
                    while i < lines.len() {
                        if lines[i].contains("autoload -Uz compinit && compinit") {
                            i += 1; // 跳过 autoload 行
                            break;
                        }
                        i += 1;
                    }
                } else {
                    // 跳过到 for f in 行之后
                    i += 1; // 跳过 marker 行
                    while i < lines.len() {
                        if lines[i].contains("for f in") && lines[i].contains(".bash") {
                            i += 1; // 跳过 for 行
                            break;
                        }
                        i += 1;
                    }
                }
                continue;
            }

            // 检查是否是独立的 fpath 行（仅在 zsh 中，且不在配置块内）
            if has_fpath && shell_info.shell_type == "zsh" && line.contains(&fpath_pattern) {
                has_fpath = false;
                i += 1; // 跳过这一行
                continue;
            }

            new_content.push_str(line);
            new_content.push('\n');
            i += 1;
        }

        // 清理末尾的多个空行
        while new_content.ends_with("\n\n") {
            new_content.pop();
        }
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }

        fs::write(&shell_info.config_file, new_content)
            .context("Failed to write to shell config file")?;

        log_success!(
            "✓ 已从 {} 中删除 completion 配置",
            shell_info.config_file.display()
        );

        Ok(())
    }
}

/// Shell 信息
struct ShellInfo {
    shell_type: String,
    completion_dir: PathBuf,
    config_file: PathBuf,
}
