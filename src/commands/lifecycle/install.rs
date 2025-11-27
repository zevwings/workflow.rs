//! 安装命令
//! 提供安装二进制文件和 shell completion 的功能

use crate::base::settings::paths::Paths;
use crate::base::shell::Detect;
#[cfg(target_os = "macos")]
use crate::base::util::remove_quarantine_attribute_with_sudo;
use crate::{log_break, log_debug, log_info, log_success, log_warning, Completion};
use anyhow::{Context, Result};
use clap_complete::shells::Shell;
use std::env;
use std::fs;
use std::path::PathBuf;
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
        log_info!("Installing shell completion scripts...");

        let shell = Detect::shell()?;
        let completion_dir = Paths::completion_dir()?;

        log_debug!("Detecting shell type...");
        log_debug!("Detected: {}", shell);

        // 创建 completion 目录
        fs::create_dir_all(&completion_dir).context("Failed to create completion directory")?;
        log_debug!("Completion directory: {}", completion_dir.display());

        // 生成 completion 脚本
        // 只生成当前检测到的 shell 类型的补全脚本
        log_debug!("Generating completion scripts...");

        let shell_type_str = shell.to_string();
        log_debug!("Generating {} completion scripts...", shell_type_str);
        Completion::generate_all_completions(
            Some(shell_type_str),
            Some(completion_dir.to_string_lossy().to_string()),
        )?;

        // 配置 shell 配置文件
        log_debug!("Configuring shell configuration file...");
        Completion::configure_shell_config(&shell)?;

        log_success!("  shell completion installation complete");
        log_break!();

        // 根据检测到的 shell 类型提示相应的重新加载命令
        let reload_hint = match shell {
            Shell::Zsh => "source ~/.zshrc",
            Shell::Bash => "source ~/.bash_profile  # or source ~/.bashrc",
            Shell::Fish => "Reopen terminal or run: source ~/.config/fish/config.fish",
            Shell::PowerShell => "Reopen PowerShell or run: . $PROFILE",
            Shell::Elvish => "Reopen terminal or run: source ~/.elvish/rc.elv",
            _ => "Please reopen terminal or reload shell configuration file",
        };
        log_info!("Hint: Please run the following command to reload configuration:");
        log_info!("  {}", reload_hint);

        Ok(())
    }

    /// 安装二进制文件到系统目录
    ///
    /// 在当前可执行文件所在目录查找 workflow 二进制文件，
    /// 并将其复制到系统二进制目录（通常是 /usr/local/bin）。
    pub fn install_binaries() -> Result<()> {
        let install_dir = Paths::binary_install_dir();
        log_info!("Installing binaries to {}...", install_dir);

        // 创建安装目录（Windows 需要）
        let install_path = PathBuf::from(&install_dir);
        fs::create_dir_all(&install_path).context("Failed to create install directory")?;

        // 获取当前可执行文件所在目录
        let current_exe = env::current_exe().context("Failed to get current executable path")?;
        let current_dir = current_exe
            .parent()
            .context("Failed to get parent directory of executable")?;

        log_debug!("Current directory: {}", current_dir.display());
        log_debug!("Install directory: {}", install_dir);

        let binaries = Paths::command_names();
        let mut installed_count = 0;

        for binary in binaries {
            let binary_name = Paths::binary_name(binary);

            let source = current_dir.join(&binary_name);
            let target = install_path.join(&binary_name);

            if !source.exists() {
                log_warning!("Binary file {} does not exist, skipping", source.display());
                continue;
            }

            log_info!("  Installing {} -> {}", binary_name, target.display());

            // Unix: 使用 sudo 复制文件
            // Windows: 直接复制文件
            if cfg!(target_os = "windows") {
                fs::copy(&source, &target).with_context(|| {
                    format!(
                        "Failed to copy {} to {}",
                        source.display(),
                        target.display()
                    )
                })?;
            } else {
                let status = Command::new("sudo")
                    .arg("cp")
                    .arg(&source)
                    .arg(&target)
                    .status()
                    .context(format!(
                        "Failed to copy {} to {}",
                        source.display(),
                        target.display()
                    ))?;

                if !status.success() {
                    anyhow::bail!("Failed to install {}", binary);
                }

                // 设置执行权限（仅 Unix）
                Command::new("sudo")
                    .arg("chmod")
                    .arg("+x")
                    .arg(&target)
                    .status()
                    .context(format!(
                        "Failed to set executable permission for {}",
                        target.display()
                    ))?;

                // 在 macOS 上，复制后立即移除目标文件的隔离属性（如果存在）
                // 使用 sudo 因为目标文件在系统目录中，需要管理员权限
                // 这确保安装后的文件不会有隔离属性，避免 Gatekeeper 阻止执行
                #[cfg(target_os = "macos")]
                {
                    log_debug!(
                        "Removing quarantine attribute from installed binary: {}",
                        target.display()
                    );
                    remove_quarantine_attribute_with_sudo(&target)?;
                }
            }

            log_success!("{} installation complete", binary_name);
            installed_count += 1;
        }

        if installed_count > 0 {
            log_success!(
                "  Binary files installation complete ({} installed)",
                installed_count
            );
            log_info!("Installed commands:");
            log_info!("  - workflow (main command with subcommands: pr, log, jira, etc.)");
        } else {
            anyhow::bail!("No installable binary files found");
        }

        Ok(())
    }
}
