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
    /// 配置 shell 配置文件以启用 completion
    pub fn configure_shell_config(shell_info: &ShellInfo) -> Result<()> {
        let config_content = fs::read_to_string(&shell_info.config_file)
            .unwrap_or_else(|_| String::new());

        let (marker, config_line) = if shell_info.shell_type == "zsh" {
            (
                "# Workflow CLI completions",
                format!(
                    "fpath=({} $fpath)\nautoload -Uz compinit && compinit",
                    shell_info.completion_dir.display()
                ),
            )
        } else {
            (
                "# Workflow CLI completions",
                format!(
                    r#"for f in {}/*.bash; do source "$f"; done"#,
                    shell_info.completion_dir.display()
                ),
            )
        };

        // 检查是否已存在配置
        if config_content.contains(marker) {
            log_success!("✓ completion 配置已存在于 {}", shell_info.config_file.display());
            return Ok(());
        }

        // 添加配置
        let mut new_content = config_content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str("\n# Workflow CLI completions\n");
        new_content.push_str(&config_line);
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

