//! Shell Completion 管理工具
//!
//! 本模块提供了 Shell Completion 的完整管理功能，包括：
//! - 生成 completion 脚本文件
//! - 配置 shell 配置文件以启用 completion
//! - 创建 completion 配置文件
//! - 删除 completion 配置和文件
//! - 获取 completion 文件列表

use crate::base::settings::paths::Paths;
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
        fs::create_dir_all(&workflow_dir).context("Failed to create .workflow directory")?;
        Ok(workflow_dir)
    }

    /// 创建并写入 workflow completion 配置文件
    ///
    /// 根据 shell 类型生成不同的配置：
    /// - zsh 和 bash：创建统一的 `~/.workflow/.completions` 配置文件
    /// - fish, powershell, elvish：返回 None（不使用统一配置文件，直接写入各自的配置文件）
    ///
    /// 注意：`_workflow` 文件包含 `workflow` 命令及其所有子命令的 completion，
    /// 包括 `github`、`proxy`、`log`、`clean` 等子命令。
    /// `_pr` 和 `_qk` 是独立命令的 completion 文件。
    fn create_completion_config_file(shell: &Shell) -> Result<Option<PathBuf>> {
        let workflow_dir = Self::create_workflow_dir()?;
        let config_file = workflow_dir.join(".completions");

        let config_content = match shell {
            Shell::Zsh => "# Workflow CLI completions\n\
                # Zsh completion setup\n\
                \n\
                fpath=($HOME/.workflow/completions $fpath)\n\
                if [[ -f $HOME/.workflow/completions/_workflow ]]; then\n\
                    source $HOME/.workflow/completions/_workflow\n\
                    source $HOME/.workflow/completions/_pr\n\
                    source $HOME/.workflow/completions/_qk\n\
                fi\n"
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

        fs::write(&config_file, config_content)
            .context("Failed to write workflow completion config file")?;

        Ok(Some(config_file))
    }

    /// 直接写入 shell 配置文件（用于 fish, powershell, elvish）
    ///
    /// 这些 shell 不使用统一配置文件，而是直接写入各自的配置文件。
    fn write_completion_to_shell_config(shell: &Shell) -> Result<()> {
        let config_path = crate::base::settings::paths::Paths::config_file(shell)?;

        // 读取现有配置
        let existing_content = if config_path.exists() {
            fs::read_to_string(&config_path).context("Failed to read shell config file")?
        } else {
            String::new()
        };

        // 检查是否已包含我们的配置
        let marker = "# Workflow CLI completions";
        if existing_content.contains(marker) {
            log_debug!("Completion 配置已存在于 {}", config_path.display());
            return Ok(());
        }

        // 生成配置内容
        let completion_content = match shell {
            Shell::Fish => {
                format!(
                    "\n{}\n\
                    source $HOME/.workflow/completions/workflow.fish\n\
                    source $HOME/.workflow/completions/pr.fish\n\
                    source $HOME/.workflow/completions/qk.fish\n",
                    marker
                )
            }
            Shell::PowerShell => {
                format!(
                    "\n{}\n\
                    . $HOME/.workflow/completions/_workflow.ps1\n\
                    . $HOME/.workflow/completions/_pr.ps1\n\
                    . $HOME/.workflow/completions/_qk.ps1\n",
                    marker
                )
            }
            Shell::Elvish => {
                format!(
                    "\n{}\n\
                    source $HOME/.workflow/completions/workflow.elv\n\
                    source $HOME/.workflow/completions/pr.elv\n\
                    source $HOME/.workflow/completions/qk.elv\n",
                    marker
                )
            }
            _ => return Ok(()), // zsh 和 bash 不使用此方法
        };

        // 追加到配置文件
        let mut new_content = existing_content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(&completion_content);

        // 确保配置文件目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create shell config directory")?;
        }

        // 写入文件
        fs::write(&config_path, new_content)
            .context("Failed to write completion to shell config file")?;

        log_debug!("已将 completion 配置写入 {}", config_path.display());

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

                // 使用 ShellConfigManager 添加 source 语句
                let source_pattern = "source $HOME/.workflow/.completions";
                let added = crate::base::shell::ShellConfigManager::add_source(
                    source_pattern,
                    Some("Workflow CLI completions"),
                )
                .context("Failed to add completion source to shell config")?;

                if !added {
                    log_success!("completion 配置已存在于 shell 配置文件");
                } else {
                    log_success!("已将 completion 配置添加到 shell 配置文件");
                }
            }
            Shell::Fish | Shell::PowerShell | Shell::Elvish => {
                // 直接写入各自的配置文件
                Self::write_completion_to_shell_config(shell)?;
                log_success!("已将 completion 配置写入 shell 配置文件");
            }
            _ => {
                anyhow::bail!("不支持的 shell 类型: {}", shell);
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
                // zsh 和 bash 使用统一配置文件，移除 source 语句
                let source_pattern = "source $HOME/.workflow/.completions";
                let removed = crate::base::shell::ShellConfigManager::remove_source(source_pattern)
                    .context("Failed to remove completion source from shell config")?;

                if !removed {
                    log_debug!("completion 配置未在 {} 配置文件中找到", shell);
                } else {
                    log_debug!("已从 {} 配置文件中删除 completion 配置", shell);
                }
            }
            Shell::Fish | Shell::PowerShell | Shell::Elvish => {
                // fish, powershell, elvish 直接写入配置文件，需要从配置文件中移除
                Self::remove_completion_block_from_config(shell)?;
            }
            _ => {
                log_debug!("不支持的 shell 类型: {}", shell);
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
                        log_debug!("移除 {} 的 completion 配置失败: {}", shell, e);
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
        let config_path = crate::base::settings::paths::Paths::config_file(shell)?;

        if !config_path.exists() {
            return Ok((false, config_path));
        }

        let content =
            fs::read_to_string(&config_path).context("Failed to read shell config file")?;

        // 检查配置标记
        let marker = "# Workflow CLI completions";
        let configured = match shell {
            Shell::Zsh | Shell::Bash => {
                // zsh 和 bash 使用统一配置文件，检查 source 语句
                let home = std::env::var("HOME").unwrap_or_default();
                let abs_path = format!("source {}/.workflow/.completions", home);
                content.contains("source $HOME/.workflow/.completions")
                    || content.contains(&abs_path)
            }
            Shell::Fish | Shell::PowerShell | Shell::Elvish => {
                // fish, powershell, elvish 直接写入配置文件
                content.contains(marker)
            }
            _ => false,
        };

        Ok((configured, config_path))
    }

    /// 从配置文件中移除 completion 配置块
    ///
    /// 用于 fish, powershell, elvish 这些直接写入配置文件的 shell。
    /// 移除配置块包括标记行和所有相关的 source 行。
    fn remove_completion_block_from_config(shell: &Shell) -> Result<()> {
        let config_path = crate::base::settings::paths::Paths::config_file(shell)?;
        if !config_path.exists() {
            log_debug!("配置文件不存在: {}", config_path.display());
            return Ok(());
        }

        let content =
            fs::read_to_string(&config_path).context("Failed to read shell config file")?;

        // 移除配置块
        let marker = "# Workflow CLI completions";
        if !content.contains(marker) {
            log_debug!("completion 配置未在 {} 配置文件中找到", shell);
            return Ok(());
        }

        // 移除配置块（包括标记和所有 source 行）
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();
        let mut skip_block = false;

        for line in lines {
            if line.contains(marker) {
                skip_block = true;
                continue;
            }

            if skip_block {
                // 检查是否是 source 行（fish, powershell, elvish 的格式）
                if line.trim().starts_with("source")
                    || line.trim().starts_with(".")
                    || line.trim().contains(".workflow/completions")
                {
                    continue;
                }

                // 如果遇到空行，停止跳过
                if line.trim().is_empty() {
                    skip_block = false;
                    // 不添加这个空行，避免多余的空行
                    continue;
                }

                // 如果遇到非空行且不是 source 行，停止跳过
                skip_block = false;
            }

            new_lines.push(line);
        }

        // 清理末尾的多个空行
        while new_lines.last().map(|s| s.trim().is_empty()) == Some(true) {
            new_lines.pop();
        }

        let new_content = new_lines.join("\n");
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            fs::write(&config_path, format!("{}\n", new_content))
                .context("Failed to write shell config file")?;
        } else {
            fs::write(&config_path, new_content).context("Failed to write shell config file")?;
        }

        log_debug!("已从 {} 配置文件中删除 completion 配置", shell);
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
    /// 返回独立的 completion 文件列表：
    /// - `_workflow` / `workflow.bash`: 包含 `workflow` 命令及其所有子命令（包括 `github`）
    /// - `_pr` / `pr.bash`: `pr` 独立命令
    /// - `_qk` / `qk.bash`: `qk` 独立命令
    pub fn get_completion_files(shell: &Shell) -> Vec<PathBuf> {
        let completion_dir = Paths::completion_dir().unwrap_or_default();
        let commands = ["workflow", "pr", "qk"];
        let shell_type_str = shell.to_string();
        super::files::get_completion_files_for_shell(&shell_type_str, &commands)
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
        let commands = ["workflow", "pr", "qk"];
        let all_file_names = super::files::get_all_completion_files(&commands);
        let all_files: Vec<PathBuf> = all_file_names
            .iter()
            .map(|name| completion_dir.join(name))
            .collect();

        let mut removed_count = 0;
        for file in &all_files {
            if file.exists() {
                if let Err(e) = fs::remove_file(file) {
                    log_info!("删除失败: {} ({})", file.display(), e);
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
            fs::remove_file(&workflow_config_file)
                .context("Failed to remove workflow completion config file")?;
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
    /// - `workflow` 命令及其所有子命令（包括 `github`、`proxy`、`log`、`clean` 等）
    /// - `pr` 独立命令
    /// - `qk` 独立命令
    pub fn generate_all_completions(
        shell_type: Option<String>,
        output_dir: Option<String>,
    ) -> Result<()> {
        super::generate::generate_all_completions(shell_type, output_dir)
    }
}
