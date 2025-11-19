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
    /// 配置文件同时支持 zsh 和 bash
    ///
    /// 注意：`_workflow` 文件包含 `workflow` 命令及其所有子命令的 completion，
    /// 包括 `github`、`proxy`、`log`、`clean` 等子命令。
    /// `_pr` 和 `_qk` 是独立命令的 completion 文件。
    fn create_completion_config_file(_shell: &Shell) -> Result<PathBuf> {
        let completion_dir = Paths::completion_dir()?;
        let workflow_dir = Self::create_workflow_dir()?;
        let config_file = workflow_dir.join(".completions");

        // 生成同时支持 zsh 和 bash 的配置
        let config_content = format!(
            "# Workflow CLI completions\n\
            # Supports both zsh and bash\n\
            \n\
            # Zsh completion setup\n\
            if [[ -n \"$ZSH_VERSION\" ]]; then\n\
                fpath=({} $fpath)\n\
                if [[ -f {}/_workflow ]]; then\n\
                    source {}/_workflow\n\
                    source {}/_pr\n\
                    source {}/_qk\n\
                fi\n\
            fi\n\
            \n\
            # Bash completion setup\n\
            if [[ -n \"$BASH_VERSION\" ]]; then\n\
                for f in {}/*.bash; do\n\
                    [[ -f \"$f\" ]] && source \"$f\"\n\
                done\n\
            fi\n",
            completion_dir.display(),
            completion_dir.display(),
            completion_dir.display(),
            completion_dir.display(),
            completion_dir.display(),
            completion_dir.display()
        );

        fs::write(&config_file, config_content)
            .context("Failed to write workflow completion config file")?;

        Ok(config_file)
    }

    /// 配置 shell 配置文件以启用 completion
    pub fn configure_shell_config(shell: &Shell) -> Result<()> {
        let config_file = Paths::config_file(shell)?;

        // 创建 workflow completion 配置文件
        let workflow_config_file = Self::create_completion_config_file(shell)?;
        let workflow_config_file_str = workflow_config_file.display().to_string();

        // 读取 shell 配置文件
        let config_content = fs::read_to_string(&config_file).unwrap_or_else(|_| String::new());

        // 检查是否已经引用了 workflow 配置文件
        let source_pattern = "source $HOME/.workflow/.completions";

        // 也检查是否使用了绝对路径
        let source_pattern_abs = format!("source {}", workflow_config_file_str);

        if config_content.contains(source_pattern) || config_content.contains(&source_pattern_abs) {
            log_success!("completion 配置已存在于 {}", config_file.display());
            return Ok(());
        }

        // 添加 source 语句到 shell 配置文件
        let mut new_content = config_content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str("\n# Workflow CLI completions\n");
        new_content.push_str(source_pattern);
        new_content.push('\n');
        new_content.push('\n');

        fs::write(&config_file, new_content).context("Failed to write to shell config file")?;

        log_success!("已将 completion 配置添加到 {}", config_file.display());

        Ok(())
    }

    /// 从 shell 配置文件中移除 completion 配置
    pub fn remove_completion_config(shell: &Shell) -> Result<()> {
        let config_file = Paths::config_file(shell)?;
        let config_content = fs::read_to_string(&config_file).unwrap_or_else(|_| String::new());

        // 检查是否引用了 workflow 配置文件
        let source_pattern = "source $HOME/.workflow/.completions";

        let has_source = config_content.contains(source_pattern);

        // 也检查绝对路径
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_config_file = PathBuf::from(&home).join(".workflow").join(".completions");
        let source_pattern_abs = format!("source {}", workflow_config_file.display());
        let has_source_abs = config_content.contains(&source_pattern_abs);

        if !has_source && !has_source_abs {
            log_info!("completion 配置未在 {} 中找到", config_file.display());
            return Ok(());
        }

        // 删除配置块（包括 marker 和 source 行）
        let marker_start = "# Workflow CLI completions";
        let mut new_content = String::new();
        let lines: Vec<&str> = config_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // 检查是否是配置块开始
            if line.contains(marker_start) {
                i += 1; // 跳过 marker 行
                        // 跳过 source 行
                while i < lines.len() {
                    let current_line = lines[i];
                    if current_line.contains(source_pattern)
                        || current_line.contains(&source_pattern_abs)
                    {
                        i += 1; // 跳过 source 行
                        break;
                    }
                    // 如果遇到空行，停止
                    if current_line.trim().is_empty() {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            // 跳过独立的 source 行（不在配置块内）
            if line.contains(source_pattern) || line.contains(&source_pattern_abs) {
                i += 1;
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

        fs::write(&config_file, new_content).context("Failed to write to shell config file")?;

        log_success!("已从 {} 中删除 completion 配置", config_file.display());

        Ok(())
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
