//! Shell Completion 管理工具
//!
//! 本模块提供了 Shell Completion 的完整管理功能，包括：
//! - 生成 completion 脚本文件
//! - 配置 shell 配置文件以启用 completion
//! - 创建 completion 配置文件
//! - 删除 completion 配置和文件
//! - 获取 completion 文件列表

use crate::base::settings::paths::Paths;
use crate::base::shell::ShellConfigManager;
use crate::log_debug;
use crate::log_info;
use crate::log_success;
use anyhow::{Context, Result};
use clap_complete::Shell;
use std::fs;
use std::path::PathBuf;

/// Completion 管理工具
///
/// 提供 Shell Completion 的配置和管理功能。
/// 支持 zsh、bash、fish、powershell、elvish 等多种 shell。
pub struct Completion;

impl Completion {
    /// 创建 workflow 配置文件目录
    fn create_workflow_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_dir = PathBuf::from(&home).join(".workflow");
        fs::create_dir_all(&workflow_dir).with_context(|| {
            format!(
                "Failed to create workflow config directory: {}",
                workflow_dir.display()
            )
        })?;
        Ok(workflow_dir)
    }

    /// 创建并写入 workflow completion 配置文件
    ///
    /// 根据 shell 类型生成不同的配置：
    /// - zsh 和 bash：创建统一的 `~/.workflow/.completions` 配置文件
    /// - fish, powershell, elvish：返回 None（不使用统一配置文件，直接写入各自的配置文件）
    ///
    /// 注意：`_workflow` 文件包含 `workflow` 命令及其所有子命令的 completion，
    /// 包括 `pr`、`log`、`jira`、`github`、`llm`、`proxy`、`log-level` 等子命令。
    fn create_completion_config_file(shell: &Shell) -> Result<Option<PathBuf>> {
        let workflow_dir = Self::create_workflow_dir()?;
        let config_file = workflow_dir.join(".completions");

        let config_content = match shell {
            Shell::Zsh => "# Workflow CLI completions\n\
                # Zsh completion setup\n\
                \n\
                fpath=($HOME/.workflow/completions $fpath)\n\
                autoload -Uz compinit\n\
                compinit\n"
                .to_string(),
            Shell::Bash => "# Workflow CLI completions\n\
                # Bash completion setup\n\
                \n\
                for f in $HOME/.workflow/completions/*.bash; do\n\
                    [[ -f \"$f\" ]] && source \"$f\"\n\
                done\n"
                .to_string(),
            // fish, powershell, elvish 不使用统一配置文件
            _ => return Ok(None),
        };

        fs::write(&config_file, config_content).with_context(|| {
            format!(
                "Failed to write workflow completion config file: {}",
                config_file.display()
            )
        })?;

        Ok(Some(config_file))
    }

    /// 配置 shell 配置文件以启用 completion（用于 fish, powershell, elvish）
    ///
    /// 这些 shell 不使用统一配置文件，而是直接写入各自的配置文件。
    /// 使用 ShellConfigManager 为 completion 文件添加 source 语句。
    fn write_completion_to_shell_config(shell: &Shell) -> Result<()> {
        // 获取每个 shell 的 completion 文件路径
        let workflow_source = match shell {
            Shell::Fish => "$HOME/.workflow/completions/workflow.fish",
            Shell::PowerShell => "$HOME/.workflow/completions/_workflow.ps1",
            Shell::Elvish => "$HOME/.workflow/completions/workflow.elv",
            _ => return Ok(()), // zsh 和 bash 不使用此方法
        };

        // 检查是否已配置
        if ShellConfigManager::has_source_for_shell(shell, workflow_source)? {
            log_debug!("Completion config already exists in {} config file", shell);
            return Ok(());
        }

        // 使用 ShellConfigManager 为 completion 文件添加 source 语句
        ShellConfigManager::add_source_for_shell(
            shell,
            workflow_source,
            Some("Workflow CLI completions"),
        )
        .with_context(|| {
            format!(
                "Failed to add workflow completion source to {} config",
                shell
            )
        })?;

        log_debug!("Completion config written to {} config file", shell);

        Ok(())
    }

    /// 配置 shell 配置文件以启用 completion
    ///
    /// 根据 shell 类型采用不同的配置策略：
    /// - zsh 和 bash：创建统一配置文件并添加到 shell 配置文件
    /// - fish, powershell, elvish：直接写入各自的配置文件
    pub fn configure_shell_config(shell: &Shell) -> Result<()> {
        match shell {
            Shell::Zsh | Shell::Bash => {
                // 创建 workflow completion 配置文件
                let _workflow_config_file = Self::create_completion_config_file(shell)?;

                // 使用 ShellConfigManager 添加 source 语句（指定 shell 类型）
                let source_pattern = "$HOME/.workflow/.completions";
                let added = ShellConfigManager::add_source_for_shell(
                    shell,
                    source_pattern,
                    Some("Workflow CLI completions"),
                )
                .with_context(|| {
                    format!("Failed to add completion source to {} config file", shell)
                })?;

                if !added {
                    log_success!("Completion config already exists in {} config file", shell);
                } else {
                    log_success!("Completion config added to {} config file", shell);
                }
            }
            Shell::Fish | Shell::PowerShell | Shell::Elvish => {
                // 直接写入各自的配置文件
                Self::write_completion_to_shell_config(shell)?;
                log_success!("Completion config written to shell config file");
            }
            _ => {
                anyhow::bail!("Unsupported shell type: {}. Supported shell types: zsh, bash, fish, powershell, elvish", shell);
            }
        }

        Ok(())
    }

    /// 从 shell 配置文件中移除 completion 配置
    ///
    /// 根据 shell 类型采用不同的移除策略：
    /// - zsh 和 bash：移除 source 语句
    /// - fish, powershell, elvish：从配置文件中移除配置块
    pub fn remove_completion_config(shell: &Shell) -> Result<()> {
        match shell {
            Shell::Zsh | Shell::Bash => {
                // zsh 和 bash 使用统一配置文件，移除 source 语句（指定 shell 类型）
                let source_pattern = "$HOME/.workflow/.completions";
                let removed = ShellConfigManager::remove_source_for_shell(shell, source_pattern)
                    .with_context(|| {
                        format!(
                            "Failed to remove completion source from {} config file",
                            shell
                        )
                    })?;

                if !removed {
                    log_debug!("Completion config not found in {} config file", shell);
                } else {
                    log_debug!("Completion config removed from {} config file", shell);
                }
            }
            Shell::Fish | Shell::PowerShell | Shell::Elvish => {
                // fish, powershell, elvish 直接写入配置文件，需要从配置文件中移除
                Self::remove_completion_block_from_config(shell)?;
            }
            _ => {
                log_debug!("Unsupported shell type: {}", shell);
            }
        }

        Ok(())
    }

    /// 移除所有 shell 的 completion 配置
    ///
    /// 遍历所有支持的 shell 类型，移除已配置的 completion。
    pub fn remove_all_completion_configs() -> Result<()> {
        let all_shells = vec![
            Shell::Zsh,
            Shell::Bash,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Elvish,
        ];

        for shell in &all_shells {
            // 检查是否已配置
            match Self::is_shell_configured_for_removal(shell) {
                Ok(true) => {
                    if let Err(e) = Self::remove_completion_config(shell) {
                        log_debug!("Failed to remove completion config for {}: {}", shell, e);
                    }
                }
                Ok(false) => {
                    // 未配置，跳过
                }
                Err(_) => {
                    // 检查失败，跳过
                }
            }
        }

        Ok(())
    }

    /// 检查 shell 是否已配置 completion
    ///
    /// 返回配置状态和配置文件路径。
    ///
    /// # 返回
    ///
    /// `Result<(bool, PathBuf)>` - (是否已配置, 配置文件路径)
    pub fn is_shell_configured(shell: &Shell) -> Result<(bool, PathBuf)> {
        let config_path = Paths::config_file(shell)?;

        if !config_path.exists() {
            return Ok((false, config_path));
        }

        // 检查配置标记
        let configured = match shell {
            Shell::Zsh | Shell::Bash => {
                // zsh 和 bash 使用统一配置文件，检查 source 语句
                let source_pattern = "$HOME/.workflow/.completions";
                ShellConfigManager::has_source_for_shell(shell, source_pattern).unwrap_or(false)
            }
            Shell::Fish | Shell::PowerShell | Shell::Elvish => {
                // fish, powershell, elvish 直接写入配置文件，检查第一个 completion 文件
                let workflow_source = match shell {
                    Shell::Fish => "$HOME/.workflow/completions/workflow.fish",
                    Shell::PowerShell => "$HOME/.workflow/completions/_workflow.ps1",
                    Shell::Elvish => "$HOME/.workflow/completions/workflow.elv",
                    _ => return Ok((false, config_path)),
                };
                ShellConfigManager::has_source_for_shell(shell, workflow_source).unwrap_or(false)
            }
            _ => false,
        };

        Ok((configured, config_path))
    }

    /// 从配置文件中移除 completion 配置（用于 fish, powershell, elvish）
    ///
    /// 使用 ShellConfigManager 移除每个 completion 文件的 source 语句。
    fn remove_completion_block_from_config(shell: &Shell) -> Result<()> {
        // 获取每个 shell 的 completion 文件路径
        let workflow_source = match shell {
            Shell::Fish => "$HOME/.workflow/completions/workflow.fish",
            Shell::PowerShell => "$HOME/.workflow/completions/_workflow.ps1",
            Shell::Elvish => "$HOME/.workflow/completions/workflow.elv",
            _ => return Ok(()), // zsh 和 bash 不使用此方法
        };

        // 使用 ShellConfigManager 移除 completion 文件的 source 语句
        let removed = ShellConfigManager::remove_source_for_shell(shell, workflow_source)
            .with_context(|| {
                format!(
                    "Failed to remove workflow completion source from {} config",
                    shell
                )
            })?;

        if removed {
            log_debug!("Completion config removed from {} config file", shell);
        } else {
            log_debug!("Completion config not found in {} config file", shell);
        }

        Ok(())
    }

    /// 检查 shell 是否已配置 completion（用于移除）
    ///
    /// 这是一个简化版本，只检查是否存在配置，不返回路径。
    fn is_shell_configured_for_removal(shell: &Shell) -> Result<bool> {
        Ok(Self::is_shell_configured(shell)?.0)
    }

    /// 获取 completion 文件列表（根据 shell 类型）
    ///
    /// 返回 completion 文件列表：
    /// - `_workflow` / `workflow.bash`: 包含 `workflow` 命令及其所有子命令（包括 `pr`、`log`、`jira`、`github`、`llm` 等）
    pub fn get_completion_files(shell: &Shell) -> Vec<PathBuf> {
        let completion_dir = Paths::completion_dir().unwrap_or_default();
        let commands = Paths::command_names();
        let shell_type_str = shell.to_string();
        super::helpers::get_completion_files_for_shell(&shell_type_str, commands)
            .unwrap_or_default()
            .iter()
            .map(|name| completion_dir.join(name))
            .collect()
    }

    /// 删除 completion 文件
    ///
    /// 删除所有 shell 类型的 completion 文件（zsh, bash, fish, powershell, elvish），
    /// 确保卸载时完全清理所有可能存在的 completion 文件。
    pub fn remove_completion_files(_shell: &Shell) -> Result<usize> {
        let completion_dir = Paths::completion_dir()?;
        // 获取所有 shell 类型的 completion 文件
        let commands = Paths::command_names();
        let all_file_names = super::helpers::get_all_completion_files(commands);
        let all_files: Vec<PathBuf> = all_file_names
            .iter()
            .map(|name| completion_dir.join(name))
            .collect();

        let mut removed_count = 0;
        for file in &all_files {
            if file.exists() {
                if let Err(e) = fs::remove_file(file) {
                    log_info!("Failed to delete: {} ({})", file.display(), e);
                } else {
                    log_info!("  Removed: {}", file.display());
                    removed_count += 1;
                }
            }
        }

        if removed_count > 0 {
            log_info!("  Completion script files removed");
        } else {
            log_debug!("  Completion script files not found (may not be installed)");
        }

        Ok(removed_count)
    }

    /// 删除 workflow completion 配置文件
    pub fn remove_completion_config_file() -> Result<()> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_config_file = PathBuf::from(&home).join(".workflow").join(".completions");

        if workflow_config_file.exists() {
            fs::remove_file(&workflow_config_file).with_context(|| {
                format!(
                    "Failed to remove workflow completion config file: {}",
                    workflow_config_file.display()
                )
            })?;
            log_info!("  Removed: {}", workflow_config_file.display());
        } else {
            log_info!(
                "  Completion config file not found: {}",
                workflow_config_file.display()
            );
        }

        Ok(())
    }

    /// 生成所有 completion 脚本文件
    ///
    /// 为所有命令生成 completion 脚本：
    /// - `workflow` 命令及其所有子命令（包括 `pr`、`log`、`jira`、`github`、`llm`、`proxy`、`log-level` 等）
    pub fn generate_all_completions(
        shell_type: Option<String>,
        output_dir: Option<String>,
    ) -> Result<()> {
        super::generate::CompletionGenerator::new(shell_type, output_dir)?.generate_all()
    }
}
