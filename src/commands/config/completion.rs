//! Completion 管理命令
//! 提供生成和管理 shell completion 脚本的功能

use std::path::PathBuf;

use clap_complete::shells::Shell;
use color_eyre::{eyre::WrapErr, Result};

use crate::config::settings::paths::Paths;
use crate::core::shell::detect::Detect;
use crate::core::shell::paths::config_file;
use crate::{br, debug, info, success, warning, Completion};

/// Shell 配置状态
#[derive(Debug, Clone)]
struct ShellStatus {
    shell: Shell,
    installed: bool,
    configured: bool,
    config_path: PathBuf,
}

/// Completion 管理命令
pub struct CompletionCommand;

impl CompletionCommand {
    /// 检查 completion 状态
    ///
    /// 检测系统中已安装的 shell 和已配置 completion 的 shell。
    pub fn check() -> Result<()> {
        info!("Checking shell completion status...");
        br!();

        // 检测当前使用的 shell
        let current_shell = Detect::shell().ok();
        debug!("Current shell: {:?}", current_shell);

        // 检测已安装的 shell
        let installed_shells = Detect::installed_shells();
        debug!("Detected installed shells: {:?}", installed_shells);

        // 检查所有支持的 shell（不仅仅是已安装的）
        let all_shells = vec![
            Shell::Zsh,
            Shell::Bash,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Elvish,
        ];

        let mut statuses = Vec::new();
        for shell in &all_shells {
            let installed = installed_shells.contains(shell);
            let (configured, config_path) = Completion::is_shell_configured(shell)
                .unwrap_or_else(|_| (false, config_file(shell).unwrap_or_default()));

            statuses.push(ShellStatus {
                shell: *shell,
                installed,
                configured,
                config_path,
            });
        }

        // 显示当前 shell 状态
        if let Some(current) = current_shell {
            if let Some(status) = statuses.iter().find(|s| s.shell == current) {
                info!("Current shell:");
                let config_status = if status.configured {
                    format!("Completion configured ({})", status.config_path.display())
                } else {
                    "Completion not configured".to_string()
                };
                info!("  {} - {}", status.shell, config_status);
                br!();

                // 如果当前 shell 未配置，显示警告
                if !status.configured {
                    warning!(
                        "Your current shell ({}) does not have completion configured",
                        current
                    );
                    info!("Hint: Run `workflow completion generate` to generate completion");
                    br!();
                }
            }
        }

        // 显示其他已安装的 shell（仅供参考，不发出警告）
        let other_shells: Vec<_> = statuses
            .iter()
            .filter(|s| s.installed && Some(s.shell) != current_shell)
            .collect();

        if !other_shells.is_empty() {
            info!("Other installed shells:");
            for status in &other_shells {
                let config_status = if status.configured {
                    format!("Completion configured ({})", status.config_path.display())
                } else {
                    "Completion not configured".to_string()
                };
                info!("  {} - {}", status.shell, config_status);
            }
            br!();
        }

        // 显示未安装但已配置的 shell（可能用户手动配置了）
        let uninstalled_configured: Vec<_> =
            statuses.iter().filter(|s| !s.installed && s.configured).collect();

        if !uninstalled_configured.is_empty() {
            info!("Uninstalled but configured shells:");
            for status in &uninstalled_configured {
                info!(
                    "  {} - Completion configured ({})",
                    status.shell,
                    status.config_path.display()
                );
            }
            br!();
        }

        // 显示当前 shell 的最终状态
        if let Some(current) = current_shell {
            if let Some(status) = statuses.iter().find(|s| s.shell == current) {
                if status.configured {
                    success!("Current shell ({}) has completion configured", current);
                }
            }
        }

        Ok(())
    }

    /// 移除 completion 配置
    ///
    /// 交互式选择要移除的 shell completion 配置。
    pub fn remove() -> Result<()> {
        info!("Removing shell completion configuration...");
        br!();

        // 检查所有支持的 shell 的配置状态
        let all_shells = vec![
            Shell::Zsh,
            Shell::Bash,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Elvish,
        ];

        let mut configured_shells = Vec::new();
        let mut shell_statuses = Vec::new();

        for shell in &all_shells {
            match Completion::is_shell_configured(shell) {
                Ok((configured, config_path)) => {
                    if configured {
                        configured_shells.push(*shell);
                        shell_statuses.push((*shell, config_path));
                    }
                }
                Err(_) => {
                    // 忽略错误，继续检查其他 shell
                }
            }
        }

        if configured_shells.is_empty() {
            info!("No configured completion found");
            return Ok(());
        }

        // 构建选项列表
        let options: Vec<String> = shell_statuses
            .iter()
            .map(|(shell, path)| format!("{} ({})", shell, path.display()))
            .collect();

        info!("Detected the following shells with completion configured:");
        for (i, option) in options.iter().enumerate() {
            info!("  [{}] {}", i, option);
        }
        br!();

        // 使用 MultiSelect 让用户选择
        let options_vec: Vec<String> = options.to_vec();
        let selected_items = crate::multiselect!(
            "Select completion to remove (use space to select, Enter to confirm, Esc to cancel)",
            options_vec
        )
        .prompt()
        .wrap_err("Failed to get user selection")?;

        let selections: Vec<usize> = options
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| {
                if selected_items.contains(opt) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        if selections.is_empty() {
            info!("No items selected, operation cancelled");
            return Ok(());
        }

        br!();
        info!("Selected the following shells:");
        for &idx in &selections {
            info!("  - {}", options[idx]);
        }
        br!();

        // 确认删除
        let confirm_msg = format!(
            "Confirm deletion of {} selected completion configurations?",
            selections.len()
        );
        if !crate::confirm!(confirm_msg).default(false).prompt()? {
            return Ok(());
        }

        br!();

        // 移除选中的配置
        let mut success_count = 0;
        let mut fail_count = 0;

        for &idx in &selections {
            let (shell, _config_path) = &shell_statuses[idx];
            info!("Removing {} completion configuration...", shell);

            match Completion::remove_completion_config(shell) {
                Ok(_) => {
                    success!("  {} completion configuration removed", shell);
                    success_count += 1;
                }
                Err(e) => {
                    warning!(
                        "  Failed to remove {} completion configuration: {}",
                        shell,
                        e
                    );
                    fail_count += 1;
                }
            }
        }

        br!();

        if success_count > 0 {
            success!(
                "Successfully removed {} completion configurations",
                success_count
            );
        }
        if fail_count > 0 {
            warning!("{} completion configuration removals failed", fail_count);
        }

        Ok(())
    }

    /// 生成 completion 脚本
    ///
    /// 自动检测当前 shell 类型，生成对应的 completion 脚本并应用到配置文件。
    /// 行为与安装流程完全一致。
    pub fn generate() -> Result<()> {
        info!("Generating shell completion scripts...");

        // 1. 自动检测当前 shell 类型（使用 Detect::shell()）
        let shell = Detect::shell().wrap_err("Failed to detect current shell type")?;
        debug!("Detected shell type: {}", shell);

        let completion_dir = Paths::completion_dir()?;
        debug!("Completion directory: {}", completion_dir.display());

        // 2. 生成 completion 脚本（与安装流程一致）
        let shell_type_str = shell.to_string();
        debug!("Generating {} completion scripts...", shell_type_str);
        Completion::generate_all_completions(
            Some(shell_type_str),
            Some(completion_dir.to_string_lossy().to_string()),
        )?;

        // 3. 应用到对应的 shell 配置文件
        debug!("Configuring shell configuration file...");
        let config_result = Completion::configure_shell_config(&shell)?;

        if config_result.already_exists {
            success!(
                "Completion config already exists in {} config file",
                config_result.shell
            );
        } else if config_result.added {
            success!(
                "Completion config added to {} config file",
                config_result.shell
            );
        } else {
            success!("Completion config written to shell config file");
        }

        success!("  shell completion generation complete");
        br!();

        // 根据检测到的 shell 类型提示相应的重新加载命令
        let reload_hint = match shell {
            Shell::Zsh => "source ~/.zshrc",
            Shell::Bash => "source ~/.bash_profile  # or source ~/.bashrc",
            Shell::Fish => "Reopen terminal or run: source ~/.config/fish/config.fish",
            Shell::PowerShell => "Reopen PowerShell or run: . $PROFILE",
            Shell::Elvish => "Reopen terminal or run: source ~/.elvish/rc.elv",
            _ => "Please reopen terminal or reload shell configuration file",
        };

        info!("Hint: Please run the following command to reload configuration:");
        info!("  {}", reload_hint);

        Ok(())
    }
}
