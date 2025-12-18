//! 安装命令
//! 提供安装二进制文件和 shell completion 的功能

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use clap_complete::shells::Shell;
use color_eyre::{
    eyre::{ContextCompat, WrapErr},
    Result,
};

use crate::base::settings::paths::Paths;
use crate::base::shell::Detect;
use crate::base::util::directory::DirectoryWalker;
use crate::{log_break, log_debug, log_info, log_success, log_warning, Completion};

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
        DirectoryWalker::new(&completion_dir).ensure_exists()?;
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
        let config_result = Completion::configure_shell_config(&shell)?;

        if config_result.already_exists {
            log_success!(
                "Completion config already exists in {} config file",
                config_result.shell
            );
        } else if config_result.added {
            log_success!(
                "Completion config added to {} config file",
                config_result.shell
            );
        } else {
            log_success!("Completion config written to shell config file");
        }

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
        DirectoryWalker::new(&install_path).ensure_exists()?;

        // 获取当前可执行文件所在目录
        let current_exe = env::current_exe().wrap_err("Failed to get current executable path")?;
        let current_dir =
            current_exe.parent().wrap_err("Failed to get parent directory of executable")?;

        log_debug!("Current directory: {}", current_dir.display());
        log_debug!("Install directory: {}", install_dir);

        let binaries = Paths::command_names();
        let mut installed_count = 0;

        for binary in binaries {
            let binary_name = Paths::binary_name(binary);

            let source = current_dir.join(&binary_name);
            let target = install_path.join(&binary_name);

            if !source.exists() {
                log_warning!(
                    "⚠  Binary file {} does not exist, skipping",
                    source.display()
                );
                continue;
            }

            log_info!("  Installing {} -> {}", binary_name, target.display());

            // Unix: 使用 sudo 复制文件
            // Windows: 直接复制文件
            if cfg!(target_os = "windows") {
                fs::copy(&source, &target).wrap_err_with(|| {
                    format!(
                        "Failed to copy {} to {}",
                        source.display(),
                        target.display()
                    )
                })?;
            } else {
                let status =
                    Command::new("sudo").arg("cp").arg(&source).arg(&target).status().wrap_err(
                        format!(
                            "Failed to copy {} to {}",
                            source.display(),
                            target.display()
                        ),
                    )?;

                if !status.success() {
                    color_eyre::eyre::bail!("Failed to install {}", binary);
                }

                // 设置执行权限（仅 Unix）
                Command::new("sudo").arg("chmod").arg("+x").arg(&target).status().wrap_err(
                    format!(
                        "Failed to set executable permission for {}",
                        target.display()
                    ),
                )?;
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
            color_eyre::eyre::bail!("No installable binary files found");
        }

        Ok(())
    }
}
