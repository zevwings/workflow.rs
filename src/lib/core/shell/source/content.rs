//! Source 语句内容处理
//!
//! 提供 source 语句的内容匹配、移除和追加功能。

use clap_complete::Shell;
use color_eyre::Result;

use crate::core::shell::block;
use crate::core::shell::paths;

/// 检查内容中是否包含 source 语句（指定 shell 类型）
///
/// 支持不同 shell 的 source 语句格式（PowerShell 使用 `.`，其他使用 `source`）。
pub fn has_source_in_content_for_shell(
    content: &str,
    shell: &Shell,
    source_path: &str,
) -> Result<bool> {
    let source_keyword = get_source_keyword(shell);

    // 构建要检查的路径列表（相对路径和绝对路径）
    let mut paths_to_check = vec![source_path.to_string()];
    if source_path.contains("$HOME") {
        let home = paths::home_dir()?;
        let abs_path = source_path.replace("$HOME", &home.to_string_lossy());
        paths_to_check.push(abs_path);
    }

    // 检查每个路径的 source 语句模式（支持单个和多个空格）
    for path in &paths_to_check {
        let patterns = [
            format!("{} {}", source_keyword, path),
            format!("{}  {}", source_keyword, path), // 支持多个空格
        ];

        for pattern in &patterns {
            if content.contains(pattern) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// 从内容中移除 source 语句（指定 shell 类型）
///
/// 移除 source 语句及其相关的注释块（如果存在）。
/// 支持不同 shell 的 source 语句格式（PowerShell 使用 `.`，其他使用 `source`）。
pub fn remove_source_from_content_for_shell(
    content: &str,
    shell: &Shell,
    source_path: &str,
) -> Result<String> {
    let home = paths::home_dir()?;
    let abs_path = source_path.replace("$HOME", &home.to_string_lossy());
    let source_keyword = get_source_keyword(shell);
    Ok(remove_source_from_content_impl(
        content,
        source_path,
        &abs_path,
        Some(source_keyword),
    ))
}

/// 检查行是否匹配 source 语句
///
/// 检查给定行是否包含指定的 source 路径，支持不同的 source 关键字。
pub fn matches_source_line(
    line: &str,
    source_keyword: Option<&str>,
    source_path: &str,
    abs_path: &str,
) -> bool {
    let contains_path = line.contains(source_path) || line.contains(abs_path);

    if let Some(keyword) = source_keyword {
        // 如果提供了关键字，检查是否以关键字开头且包含路径
        line.trim().starts_with(keyword) && contains_path
    } else {
        // 否则只检查是否包含路径
        contains_path
    }
}

/// 从内容中移除 source 语句的内部实现
///
/// 移除 source 语句及其相关的注释块（如果存在）。
/// 如果提供了 `source_keyword`，则使用它来匹配 source 语句；否则匹配任何包含路径的行。
pub fn remove_source_from_content_impl(
    content: &str,
    source_path: &str,
    abs_path: &str,
    source_keyword: Option<&str>,
) -> String {
    let lines: Vec<&str> = content.lines().collect();
    // 预分配容量，假设移除的行数不超过总行数的20%
    let estimated_capacity = (content.len() as f64 * 0.8) as usize;
    let mut new_content = String::with_capacity(estimated_capacity);
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // 检查是否是配置块开始（包含 Workflow CLI 的注释）
        if block::is_related_line(line) {
            // 跳过整个配置块
            i += 1; // 跳过注释行
                    // 查找并跳过 source 行
            while i < lines.len() {
                let current_line = lines[i];

                if matches_source_line(current_line, source_keyword, source_path, abs_path) {
                    i += 1; // 跳过 source 行
                            // 跳过后续的空行
                    while i < lines.len() && lines[i].trim().is_empty() {
                        i += 1;
                    }
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
        if matches_source_line(line, source_keyword, source_path, abs_path) {
            i += 1;
            // 跳过后续的空行
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
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

    new_content
}

/// 追加 source 语句到内容末尾
///
/// 在内容末尾追加 source 语句，包括可选的注释。
/// 使用预分配的字符串缓冲区以提高性能。
pub fn append_source_statement(
    content: &str,
    source_keyword: &str,
    source_path: &str,
    comment: Option<&str>,
) -> String {
    // 估算容量：原内容 + 注释（如果有）+ source 语句 + 换行符
    let comment_len = comment.map(|c| c.len() + 4).unwrap_or(0); // "# " + comment + "\n"
    let source_len = source_keyword.len() + source_path.len() + 3; // keyword + " " + path + "\n\n"
    let estimated_capacity = content.len() + comment_len + source_len + 1; // +1 for potential trailing newline

    let mut new_content = String::with_capacity(estimated_capacity);
    new_content.push_str(content);

    // 确保内容以换行符结尾
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    // 添加注释（如果有）
    if let Some(comment_text) = comment {
        new_content.push_str("# ");
        new_content.push_str(comment_text);
        new_content.push('\n');
    }

    // 添加 source 语句
    new_content.push_str(source_keyword);
    new_content.push(' ');
    new_content.push_str(source_path);
    new_content.push('\n');
    new_content.push('\n');

    new_content
}

/// 获取 shell 的 source 语句关键字
///
/// 不同 shell 使用不同的关键字来加载脚本：
/// - zsh, bash, fish, elvish: `source`
/// - powershell: `.`
pub(crate) fn get_source_keyword(shell: &Shell) -> &'static str {
    match shell {
        Shell::PowerShell => ".",
        _ => "source",
    }
}
