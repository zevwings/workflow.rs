//! Shell Completion 管理工具
//! 提供 Completion 配置管理、文件管理等功能

use crate::log_info;
use crate::log_success;
use crate::ShellInfo;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Completion 管理工具
pub struct Completion;

impl Completion {
    /// 创建 workflow 配置文件目录
    fn create_workflow_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_dir = PathBuf::from(&home).join(".workflow");
        fs::create_dir_all(&workflow_dir)
            .context("Failed to create .workflow directory")?;
        Ok(workflow_dir)
    }

    /// 创建并写入 workflow completion 配置文件
    /// 配置文件同时支持 zsh 和 bash
    fn create_completion_config_file(shell_info: &ShellInfo) -> Result<PathBuf> {
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
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display(),
            shell_info.completion_dir.display()
        );

        fs::write(&config_file, config_content)
            .context("Failed to write workflow completion config file")?;

        Ok(config_file)
    }

    /// 配置 shell 配置文件以启用 completion
    pub fn configure_shell_config(shell_info: &ShellInfo) -> Result<()> {
        // 创建 workflow completion 配置文件
        let workflow_config_file = Self::create_completion_config_file(shell_info)?;
        let workflow_config_file_str = workflow_config_file.display().to_string();

        // 读取 shell 配置文件
        let config_content =
            fs::read_to_string(&shell_info.config_file).unwrap_or_else(|_| String::new());

        // 检查是否已经引用了 workflow 配置文件
        let source_pattern = "source $HOME/.workflow/.completions";

        // 也检查是否使用了绝对路径
        let source_pattern_abs = format!("source {}", workflow_config_file_str);

        if config_content.contains(source_pattern) || config_content.contains(&source_pattern_abs) {
            log_success!(
                "✓ completion 配置已存在于 {}",
                shell_info.config_file.display()
            );
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

        fs::write(&shell_info.config_file, new_content)
            .context("Failed to write to shell config file")?;

        log_success!(
            "✓ 已将 completion 配置添加到 {}",
            shell_info.config_file.display()
        );

        Ok(())
    }

    /// 从 shell 配置文件中移除 completion 配置
    pub fn remove_completion_config(shell_info: &ShellInfo) -> Result<()> {
        let config_content =
            fs::read_to_string(&shell_info.config_file).unwrap_or_else(|_| String::new());

        // 检查是否引用了 workflow 配置文件
        let source_pattern = "source $HOME/.workflow/.completions";

        let has_source = config_content.contains(source_pattern);

        // 也检查绝对路径
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let workflow_config_file = PathBuf::from(&home).join(".workflow").join(".completions");
        let source_pattern_abs = format!("source {}", workflow_config_file.display());
        let has_source_abs = config_content.contains(&source_pattern_abs);

        if !has_source && !has_source_abs {
            log_info!(
                "ℹ  completion 配置未在 {} 中找到",
                shell_info.config_file.display()
            );
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
                    if current_line.contains(source_pattern) || current_line.contains(&source_pattern_abs) {
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

        fs::write(&shell_info.config_file, new_content)
            .context("Failed to write to shell config file")?;

        log_success!(
            "✓ 已从 {} 中删除 completion 配置",
            shell_info.config_file.display()
        );

        Ok(())
    }

    /// 获取 completion 文件列表（根据 shell 类型）
    pub fn get_completion_files(shell_info: &ShellInfo) -> Vec<PathBuf> {
        if shell_info.shell_type == "zsh" {
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
        }
    }

    /// 删除 completion 文件
    pub fn remove_completion_files(shell_info: &ShellInfo) -> Result<usize> {
        let completion_files = Self::get_completion_files(shell_info);

        let mut removed_count = 0;
        for file in &completion_files {
            if file.exists() {
                if let Err(e) = fs::remove_file(file) {
                    log_info!("⚠  删除失败: {} ({})", file.display(), e);
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

        Ok(removed_count)
    }
}
