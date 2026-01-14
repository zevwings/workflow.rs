//! Shell Source 语句管理
//!
//! 提供 shell 配置文件中的 source 语句管理功能，包括：
//! - 添加 source 语句
//! - 移除 source 语句
//! - 检查 source 语句是否存在
//!
//! 支持不同 shell 的 source 语句格式（PowerShell 使用 `.`，其他使用 `source`）。

mod content;

use clap_complete::Shell;
use color_eyre::Result;

use crate::core::shell::file;
use crate::core::shell::paths;

use content::{
    append_source_statement, has_source_in_content_for_shell, remove_source_from_content_for_shell,
};

/// 添加 source 语句（指定 shell 类型）
///
/// 在指定 shell 类型的配置文件中添加 source 语句。如果已存在则跳过。
/// 根据 shell 类型自动使用正确的关键字（PowerShell 使用 `.`，其他使用 `source`）。
///
/// # 参数
///
/// * `shell` - Shell 类型
/// * `source_path` - source 文件路径（支持相对路径如 `$HOME/.workflow/.completions` 或绝对路径）
/// * `comment` - 可选的注释文本
///
/// # 错误
///
/// 如果读取或写入配置文件失败，返回相应的错误信息。
pub fn add_source_for_shell(
    shell: &Shell,
    source_path: &str,
    comment: Option<&str>,
) -> Result<bool> {
    let config_path = paths::config_file(shell)?;
    let content = file::read_config_file(&config_path).unwrap_or_default();

    // 检查是否已存在（支持不同格式）
    if has_source_in_content_for_shell(&content, shell, source_path)? {
        return Ok(false);
    }

    // 添加 source 语句
    let source_keyword = content::get_source_keyword(shell);
    let new_content = append_source_statement(&content, source_keyword, source_path, comment);

    file::write_config_file(&config_path, &new_content)?;

    Ok(true)
}

/// 移除 source 语句（指定 shell 类型）
///
/// 从指定 shell 类型的配置文件中移除指定的 source 语句。
/// 支持不同 shell 的 source 语句格式（PowerShell 使用 `.`，其他使用 `source`）。
///
/// # 参数
///
/// * `shell` - Shell 类型
/// * `source_path` - source 文件路径
///
/// # 返回
///
/// 返回 `true` 如果移除了 source 语句，否则返回 `false`。
///
/// # 错误
///
/// 如果读取或写入配置文件失败，返回相应的错误信息。
pub fn remove_source_for_shell(shell: &Shell, source_path: &str) -> Result<bool> {
    let config_path = paths::config_file(shell)?;
    let content = file::read_config_file(&config_path).unwrap_or_default();

    // 检查是否存在（支持不同格式）
    if !has_source_in_content_for_shell(&content, shell, source_path)? {
        return Ok(false);
    }

    // 移除 source 语句和相关注释
    let new_content = remove_source_from_content_for_shell(&content, shell, source_path)?;

    file::write_config_file(&config_path, &new_content)?;

    Ok(true)
}

/// 检查 source 语句是否存在（指定 shell 类型）
///
/// # 参数
///
/// * `shell` - Shell 类型
/// * `source_path` - source 文件路径
///
/// # 返回
///
/// 返回 `true` 如果 source 语句存在，否则返回 `false`。
///
/// # 错误
///
/// 如果读取配置文件失败，返回相应的错误信息。
pub fn has_source_for_shell(shell: &Shell, source_path: &str) -> Result<bool> {
    let config_path = paths::config_file(shell)?;
    let content = file::read_config_file(&config_path).unwrap_or_default();
    has_source_in_content_for_shell(&content, shell, source_path)
}
