//! Shell 配置文件管理
//!
//! 提供 Shell 配置文件的读写和 source 语句管理功能。

use std::fs;
use std::path::PathBuf;

use clap_complete::Shell;

use super::config_file_path;
use super::error::ShellError;

/// 添加 source 语句到 Shell 配置文件
///
/// 根据 Shell 类型，在配置文件末尾添加 source 语句。
/// 如果语句已存在，则不重复添加。
///
/// # 参数
///
/// * `shell` - Shell 类型
/// * `source_path` - 要 source 的文件路径
/// * `comment` - 可选的注释说明
///
/// # 返回
///
/// 返回是否成功添加（如果已存在则返回 false）。
pub fn add_source(
    shell: &Shell,
    source_path: &str,
    comment: Option<&str>,
) -> Result<bool, ShellError> {
    // 检查是否已存在
    if has_source(shell, source_path)? {
        return Ok(false);
    }

    let config_path = config_file_path(shell).ok_or(ShellError::HomeNotFound)?;

    // 确保配置文件目录存在
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // 读取现有内容
    let mut content = if config_path.exists() {
        fs::read_to_string(&config_path).map_err(|e| ShellError::ConfigReadFailed {
            path: config_path.clone(),
            source: e,
        })?
    } else {
        String::new()
    };

    // 确保末尾有换行
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    // 添加注释和 source 语句
    if let Some(comment_text) = comment {
        content.push_str(&format!("\n# {}\n", comment_text));
    } else {
        content.push('\n');
    }

    // 根据 Shell 类型生成 source 语句
    let source_statement = generate_source_statement(shell, source_path);
    content.push_str(&source_statement);
    content.push('\n');

    // 写入配置文件
    fs::write(&config_path, content).map_err(|e| ShellError::ConfigWriteFailed {
        path: config_path,
        source: e,
    })?;

    Ok(true)
}

/// 检查 Shell 配置文件中是否存在 source 语句
///
/// # 参数
///
/// * `shell` - Shell 类型
/// * `source_path` - 要检查的 source 路径
///
/// # 返回
///
/// 返回是否存在该 source 语句。
pub fn has_source(shell: &Shell, source_path: &str) -> Result<bool, ShellError> {
    let config_path = config_file_path(shell).ok_or(ShellError::HomeNotFound)?;

    if !config_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&config_path).map_err(|e| ShellError::ConfigReadFailed {
        path: config_path,
        source: e,
    })?;

    // 检查是否包含 source 语句
    let patterns = get_source_patterns(shell, source_path);
    Ok(patterns.iter().any(|pattern| content.contains(pattern)))
}

/// 从 Shell 配置文件中移除 source 语句
///
/// # 参数
///
/// * `shell` - Shell 类型
/// * `source_path` - 要移除的 source 路径
///
/// # 返回
///
/// 返回是否成功移除（如果不存在则返回 false）。
pub fn remove_source(shell: &Shell, source_path: &str) -> Result<bool, ShellError> {
    let config_path = config_file_path(shell).ok_or(ShellError::HomeNotFound)?;

    if !config_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&config_path).map_err(|e| ShellError::ConfigReadFailed {
        path: config_path.clone(),
        source: e,
    })?;

    // 获取所有可能的 source 模式
    let patterns = get_source_patterns(shell, source_path);

    // 过滤掉包含 source 语句的行和相关注释
    let mut new_lines: Vec<&str> = Vec::new();
    let mut skip_next_empty = false;
    let mut removed = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // 检查是否是 source 语句
        let is_source_line = patterns.iter().any(|pattern| trimmed.contains(pattern));

        if is_source_line {
            removed = true;
            skip_next_empty = true;
            // 如果前一行是相关注释，也移除它
            if let Some(last) = new_lines.last() {
                if last.contains("Workflow") || last.contains("workflow") {
                    new_lines.pop();
                }
            }
            continue;
        }

        // 跳过 source 语句后的空行
        if skip_next_empty && trimmed.is_empty() {
            skip_next_empty = false;
            continue;
        }

        skip_next_empty = false;
        new_lines.push(line);
    }

    if removed {
        // 清理末尾多余的空行
        while new_lines.last().is_some_and(|l| l.trim().is_empty()) {
            new_lines.pop();
        }

        let mut new_content = new_lines.join("\n");
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }

        fs::write(&config_path, new_content).map_err(|e| ShellError::ConfigWriteFailed {
            path: config_path,
            source: e,
        })?;
    }

    Ok(removed)
}

/// 检查 Shell 是否已配置 completion
///
/// # 参数
///
/// * `shell` - Shell 类型
/// * `source_path` - 要检查的 source 路径
///
/// # 返回
///
/// 返回 (是否已配置, 配置文件路径)。
pub fn is_configured(shell: &Shell, source_path: &str) -> Result<(bool, PathBuf), ShellError> {
    let config_path = config_file_path(shell).ok_or(ShellError::HomeNotFound)?;
    let configured = has_source(shell, source_path)?;
    Ok((configured, config_path))
}

/// 生成 source 语句
fn generate_source_statement(shell: &Shell, source_path: &str) -> String {
    match shell {
        Shell::Zsh | Shell::Bash => {
            format!("[[ -f \"{}\" ]] && source \"{}\"", source_path, source_path)
        }
        Shell::Fish => {
            format!(
                "if test -f \"{}\"\n    source \"{}\"\nend",
                source_path, source_path
            )
        }
        Shell::PowerShell => {
            format!(
                "if (Test-Path \"{}\") {{ . \"{}\" }}",
                source_path, source_path
            )
        }
        Shell::Elvish => {
            format!(
                "if (path:is-regular \"{}\") {{ eval (slurp < \"{}\") }}",
                source_path, source_path
            )
        }
        _ => format!("source \"{}\"", source_path),
    }
}

/// 获取 source 语句的匹配模式
fn get_source_patterns(shell: &Shell, source_path: &str) -> Vec<String> {
    match shell {
        Shell::Zsh | Shell::Bash => {
            vec![
                format!("source \"{}\"", source_path),
                format!("source '{}'", source_path),
                format!("source {}", source_path),
                format!(". \"{}\"", source_path),
                format!(". '{}'", source_path),
                format!(". {}", source_path),
            ]
        }
        Shell::Fish => {
            vec![
                format!("source \"{}\"", source_path),
                format!("source '{}'", source_path),
                format!("source {}", source_path),
            ]
        }
        Shell::PowerShell => {
            vec![
                format!(". \"{}\"", source_path),
                format!(". '{}'", source_path),
            ]
        }
        Shell::Elvish => {
            vec![source_path.to_string()]
        }
        _ => vec![source_path.to_string()],
    }
}
