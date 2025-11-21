//! Completion 管理命令
//! 提供生成和管理 shell completion 脚本的功能

use crate::base::settings::paths::Paths;
use crate::base::shell::Detect;
use crate::base::util::confirm;
use crate::{log_break, log_debug, log_info, log_message, log_success, log_warning, Completion};
use anyhow::{Context, Result};
use clap_complete::shells::Shell;
use dialoguer::MultiSelect;
use std::path::PathBuf;

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
        log_info!("Checking shell completion status...");
        log_break!();

        // 检测已安装的 shell
        let installed_shells = Detect::installed_shells();
        log_debug!("Detected installed shells: {:?}", installed_shells);

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
                .unwrap_or_else(|_| (false, Paths::config_file(shell).unwrap_or_default()));

            statuses.push(ShellStatus {
                shell: *shell,
                installed,
                configured,
                config_path,
            });
        }

        // 显示结果
        log_message!("Installed shells:");
        for status in &statuses {
            if status.installed {
                let icon = if status.configured { "✓" } else { "✗" };
                let config_status = if status.configured {
                    format!("Completion configured ({})", status.config_path.display())
                } else {
                    "Completion not configured".to_string()
                };
                log_message!("  {} {} - {}", icon, status.shell, config_status);
            }
        }

        log_break!();

        // 显示未安装但已配置的 shell（可能用户手动配置了）
        let uninstalled_configured: Vec<_> = statuses
            .iter()
            .filter(|s| !s.installed && s.configured)
            .collect();

        if !uninstalled_configured.is_empty() {
            log_message!("Uninstalled but configured shells:");
            for status in &uninstalled_configured {
                log_message!(
                    "  ✓ {} - Completion configured ({})",
                    status.shell,
                    status.config_path.display()
                );
            }
            log_break!();
        }

        // 显示未配置的 shell
        let unconfigured: Vec<_> = statuses
            .iter()
            .filter(|s| s.installed && !s.configured)
            .collect();

        if !unconfigured.is_empty() {
            log_warning!("Shells without completion configured:");
            for status in &unconfigured {
                log_message!("  ✗ {} - Completion not configured", status.shell);
            }
            log_break!();
            log_info!("Hint: Run `workflow completion generate` to generate completion for unconfigured shells");
        } else if statuses.iter().any(|s| s.installed && s.configured) {
            log_success!("All installed shells have completion configured");
        }

        Ok(())
    }

    /// 移除 completion 配置
    ///
    /// 交互式选择要移除的 shell completion 配置。
    pub fn remove() -> Result<()> {
        log_info!("Removing shell completion configuration...");
        log_break!();

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
            log_info!("No configured completion found");
            return Ok(());
        }

        // 构建选项列表
        let options: Vec<String> = shell_statuses
            .iter()
            .map(|(shell, path)| format!("{} ({})", shell, path.display()))
            .collect();

        log_message!("Detected the following shells with completion configured:");
        for (i, option) in options.iter().enumerate() {
            log_message!("  [{}] {}", i, option);
        }
        log_break!();

        // 使用 MultiSelect 让用户选择
        let selections = MultiSelect::new()
            .with_prompt("Select completion to remove (use space to select, Enter to confirm, Esc to cancel)")
            .items(&options)
            .interact()
            .context("Failed to get user selection")?;

        if selections.is_empty() {
            log_info!("No items selected, operation cancelled");
            return Ok(());
        }

        log_break!();
        log_message!("Selected the following shells:");
        for &idx in &selections {
            log_message!("  - {}", options[idx]);
        }
        log_break!();

        // 确认删除
        let confirm_msg = format!(
            "Confirm deletion of {} selected completion configurations?",
            selections.len()
        );
        if !confirm(&confirm_msg, false, Some("Operation cancelled"))? {
            return Ok(());
        }

        log_break!();

        // 移除选中的配置
        let mut success_count = 0;
        let mut fail_count = 0;

        for &idx in &selections {
            let (shell, _config_path) = &shell_statuses[idx];
            log_info!("Removing {} completion configuration...", shell);

            match Completion::remove_completion_config(shell) {
                Ok(_) => {
                    log_success!("  ✓ {} completion configuration removed", shell);
                    success_count += 1;
                }
                Err(e) => {
                    log_warning!(
                        "  ✗ Failed to remove {} completion configuration: {}",
                        shell,
                        e
                    );
                    fail_count += 1;
                }
            }
        }

        log_break!();

        if success_count > 0 {
            log_success!(
                "Successfully removed {} completion configurations",
                success_count
            );
        }
        if fail_count > 0 {
            log_warning!("{} completion configuration removals failed", fail_count);
        }

        Ok(())
    }

    /// 生成 completion 脚本
    ///
    /// 自动检测当前 shell 类型，生成对应的 completion 脚本并应用到配置文件。
    /// 行为与安装流程完全一致。
    pub fn generate() -> Result<()> {
        log_info!("Generating shell completion scripts...");

        // 1. 自动检测当前 shell 类型（使用 Detect::shell()）
        let shell = Detect::shell().context("Failed to detect current shell type")?;
        log_debug!("Detected shell type: {}", shell);

        let completion_dir = Paths::completion_dir()?;
        log_debug!("Completion directory: {}", completion_dir.display());

        // 2. 生成 completion 脚本（与安装流程一致）
        let shell_type_str = shell.to_string();
        log_debug!("Generating {} completion scripts...", shell_type_str);
        Completion::generate_all_completions(
            Some(shell_type_str),
            Some(completion_dir.to_string_lossy().to_string()),
        )?;

        // 3. 应用到对应的 shell 配置文件（使用 ShellConfigManager）
        log_debug!("Configuring shell configuration file...");
        Completion::configure_shell_config(&shell)?;

        log_success!("  shell completion generation complete");
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
}
