//! Completion 管理命令
//! 提供生成和管理 shell completion 脚本的功能

use crate::{
    base::settings::paths::Paths, confirm, log_break, log_debug, log_info, log_message,
    log_success, log_warning, Completion, Detect,
};
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
        log_info!("检查 shell completion 状态...");
        log_break!();

        // 检测已安装的 shell
        let installed_shells = Detect::installed_shells();
        log_debug!("检测到已安装的 shell: {:?}", installed_shells);

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
        log_message!("已安装的 shell：");
        for status in &statuses {
            if status.installed {
                let icon = if status.configured { "✓" } else { "✗" };
                let config_status = if status.configured {
                    format!("已配置 completion ({})", status.config_path.display())
                } else {
                    "未配置 completion".to_string()
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
            log_message!("未安装但已配置的 shell：");
            for status in &uninstalled_configured {
                log_message!(
                    "  ✓ {} - 已配置 completion ({})",
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
            log_warning!("未配置 completion 的 shell：");
            for status in &unconfigured {
                log_message!("  ✗ {} - 未配置 completion", status.shell);
            }
            log_break!();
            log_info!("提示：运行 `workflow completion generate` 为未配置的 shell 生成 completion");
        } else if statuses.iter().any(|s| s.installed && s.configured) {
            log_success!("所有已安装的 shell 都已配置 completion");
        }

        Ok(())
    }

    /// 移除 completion 配置
    ///
    /// 交互式选择要移除的 shell completion 配置。
    pub fn remove() -> Result<()> {
        log_info!("移除 shell completion 配置...");
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
            log_info!("未找到已配置的 completion");
            return Ok(());
        }

        // 构建选项列表
        let options: Vec<String> = shell_statuses
            .iter()
            .map(|(shell, path)| format!("{} ({})", shell, path.display()))
            .collect();

        log_message!("检测到以下 shell 已配置 completion：");
        for (i, option) in options.iter().enumerate() {
            log_message!("  [{}] {}", i, option);
        }
        log_break!();

        // 使用 MultiSelect 让用户选择
        let selections = MultiSelect::new()
            .with_prompt("选择要移除的 completion（使用空格选择，Enter 确认，Esc 取消）")
            .items(&options)
            .interact()
            .context("Failed to get user selection")?;

        if selections.is_empty() {
            log_info!("未选择任何项，操作已取消");
            return Ok(());
        }

        log_break!();
        log_message!("已选择以下 shell：");
        for &idx in &selections {
            log_message!("  - {}", options[idx]);
        }
        log_break!();

        // 确认删除
        let confirm_msg = format!("确认删除选中的 {} 个 completion 配置？", selections.len());
        if !confirm(&confirm_msg, false, Some("操作已取消"))? {
            return Ok(());
        }

        log_break!();

        // 移除选中的配置
        let mut success_count = 0;
        let mut fail_count = 0;

        for &idx in &selections {
            let (shell, _config_path) = &shell_statuses[idx];
            log_info!("正在移除 {} 的 completion 配置...", shell);

            match Completion::remove_completion_config(shell) {
                Ok(_) => {
                    log_success!("  ✓ {} 的 completion 配置已移除", shell);
                    success_count += 1;
                }
                Err(e) => {
                    log_warning!("  ✗ 移除 {} 的 completion 配置失败: {}", shell, e);
                    fail_count += 1;
                }
            }
        }

        log_break!();

        if success_count > 0 {
            log_success!("成功移除 {} 个 completion 配置", success_count);
        }
        if fail_count > 0 {
            log_warning!("{} 个 completion 配置移除失败", fail_count);
        }

        Ok(())
    }

    /// 生成 completion 脚本
    ///
    /// 自动检测当前 shell 类型，生成对应的 completion 脚本并应用到配置文件。
    /// 行为与安装流程完全一致。
    pub fn generate() -> Result<()> {
        log_info!("生成 shell completion 脚本...");

        // 1. 自动检测当前 shell 类型（使用 Detect::shell()）
        let shell = Detect::shell().context("Failed to detect current shell type")?;
        log_debug!("检测到 shell 类型: {}", shell);

        let completion_dir = Paths::completion_dir()?;
        log_debug!("Completion 目录: {}", completion_dir.display());

        // 2. 生成 completion 脚本（与安装流程一致）
        let shell_type_str = shell.to_string();
        log_debug!("正在生成 {} completion 脚本...", shell_type_str);
        Completion::generate_all_completions(
            Some(shell_type_str),
            Some(completion_dir.to_string_lossy().to_string()),
        )?;

        // 3. 应用到对应的 shell 配置文件（使用 ShellConfigManager）
        log_debug!("正在配置 shell 配置文件...");
        Completion::configure_shell_config(&shell)?;

        log_success!("  shell completion 生成完成");
        log_break!();

        // 根据检测到的 shell 类型提示相应的重新加载命令
        let reload_hint = match shell {
            Shell::Zsh => "source ~/.zshrc",
            Shell::Bash => "source ~/.bash_profile  # 或 source ~/.bashrc",
            Shell::Fish => "重新打开终端或运行: source ~/.config/fish/config.fish",
            Shell::PowerShell => "重新打开 PowerShell 或运行: . $PROFILE",
            Shell::Elvish => "重新打开终端或运行: source ~/.elvish/rc.elv",
            _ => "请重新打开终端或重新加载 shell 配置文件",
        };

        log_info!("提示：请运行以下命令重新加载配置:");
        log_info!("  {}", reload_hint);

        Ok(())
    }
}
