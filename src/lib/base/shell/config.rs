//! Shell 配置管理器
//!
//! 提供通用的 shell 配置文件管理功能，包括：
//! - 环境变量管理（export 语句）
//! - Source 语句管理
//! - 配置块管理
//!
//! 支持 zsh、bash、fish、powershell、elvish 等 shell 的配置文件。

use crate::base::settings::paths::Paths;
use crate::base::util::file::{FileReader, FileWriter};
use clap_complete::Shell;
use color_eyre::{eyre::WrapErr, Result};
use std::collections::HashMap;
use std::path::PathBuf;

use super::detect::Detect;

/// Shell 配置管理器
///
/// 提供通用的 shell 配置文件管理功能，供 Proxy 和 Completion 模块共用。
pub struct ShellConfigManager;

impl ShellConfigManager {
    // === 环境变量管理 ===

    /// 从配置块加载环境变量
    ///
    /// 从 shell 配置文件的配置块中读取环境变量。
    ///
    /// # 返回
    ///
    /// 返回环境变量 HashMap。如果配置文件不存在或没有配置块，返回空 HashMap。
    ///
    /// # 错误
    ///
    /// 如果读取配置文件失败，返回相应的错误信息。
    pub fn load_env_vars() -> Result<HashMap<String, String>> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Ok(HashMap::new());
        }

        let content = Self::read_config_file(&config_path)?;
        let (env_vars, _) = Self::parse_config_block(&content)?;

        Ok(env_vars)
    }

    /// 保存环境变量到配置块
    ///
    /// 将环境变量保存到 shell 配置文件的配置块中。
    /// 如果 key 已存在则覆盖，不存在则新增。
    ///
    /// # 参数
    ///
    /// * `env_vars` - 要保存的环境变量 HashMap
    ///
    /// # 错误
    ///
    /// 如果写入配置文件失败，返回相应的错误信息。
    pub fn save_env_vars(env_vars: &HashMap<String, String>) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // 读取现有配置
        let existing = Self::load_existing_config(&config_path)?;

        // 合并环境变量
        let merged = Self::merge_env_vars(&existing.env_in_block, env_vars);

        // 构建新内容
        let new_content = Self::build_config_content(&existing.content_without_block, &merged)?;

        // 写入文件
        Self::write_config_file(&config_path, &new_content)?;

        Ok(())
    }

    /// 批量设置环境变量
    ///
    /// 批量设置环境变量到配置块中。
    ///
    /// # 参数
    ///
    /// * `env_vars` - 要设置的环境变量 HashMap
    ///
    /// # 错误
    ///
    /// 如果写入配置文件失败，返回相应的错误信息。
    pub fn set_env_vars(env_vars: &HashMap<String, String>) -> Result<()> {
        Self::save_env_vars(env_vars)
    }

    /// 从文件中移除指定的 export 语句
    ///
    /// 从整个配置文件中移除指定的环境变量的 export 语句（包括配置块内外）。
    ///
    /// # 参数
    ///
    /// * `keys` - 要移除的环境变量键名数组
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果移除了任何内容，否则返回 `false`。
    ///
    /// # 错误
    ///
    /// 如果读取或写入配置文件失败，返回相应的错误信息。
    pub fn remove_env_vars(keys: &[&str]) -> Result<bool> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Ok(false);
        }

        let content = Self::read_config_file(&config_path)?;

        let marker_start = "# Workflow CLI Configuration - Start";
        let marker_end = "# Workflow CLI Configuration - End";

        // 使用 filter 直接过滤掉需要删除的行，避免先收集再删除
        let lines: Vec<&str> = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();

                // 跳过配置块的标记行
                if trimmed == marker_start || trimmed == marker_end {
                    return true;
                }

                // 如果行以 export 开头，检查是否匹配要删除的键
                if let Some(rest) = trimmed.strip_prefix("export ") {
                    for key in keys {
                        // 检查是否是 export KEY= 或 export KEY="
                        if let Some(after_key) = rest.strip_prefix(key) {
                            let after_key = after_key.trim_start();
                            if after_key.starts_with('=') {
                                // 找到匹配的 export 行，过滤掉
                                return false;
                            }
                        }
                    }
                }

                true
            })
            .collect();

        let removed_any = lines.len() < content.lines().count();

        if removed_any {
            // 重新构建文件内容
            let new_content = lines.join("\n");

            // 检查配置块是否为空（只有标记和注释，没有 export 语句）
            // 如果为空，则完全移除配置块
            let final_content =
                Self::remove_empty_config_block(&new_content, marker_start, marker_end);

            // 确保文件以换行符结尾
            let final_content = if final_content.ends_with('\n') {
                final_content
            } else {
                format!("{}\n", final_content)
            };

            Self::write_config_file(&config_path, &final_content)?;
        }

        Ok(removed_any)
    }

    /// 移除空的配置块
    ///
    /// 如果配置块内没有任何 export 语句（只有标记和注释），则完全移除配置块。
    fn remove_empty_config_block(content: &str, marker_start: &str, marker_end: &str) -> String {
        // 查找配置块
        if let Some(start_pos) = content.find(marker_start) {
            if let Some(end_pos) = content[start_pos..].find(marker_end) {
                let block_end = start_pos + end_pos + marker_end.len();
                let block_content = &content[start_pos + marker_start.len()..start_pos + end_pos];

                // 检查块内是否有任何 export 语句
                let has_exports = block_content.lines().any(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty()
                        && !trimmed.starts_with('#')
                        && trimmed.starts_with("export ")
                });

                // 如果没有 export 语句，移除整个配置块
                if !has_exports {
                    let before = content[..start_pos].trim_end();
                    let after = content[block_end..].trim_start();

                    return if before.is_empty() {
                        after.to_string()
                    } else if after.is_empty() {
                        format!("{}\n", before)
                    } else {
                        format!("{}\n{}", before, after)
                    };
                }
            }
        }

        content.to_string()
    }

    // === Source 语句管理 ===

    /// 添加 source 语句
    ///
    /// 在 shell 配置文件中添加 source 语句。如果已存在则跳过。
    ///
    /// # 参数
    ///
    /// * `source_path` - source 文件路径（支持相对路径如 `$HOME/.workflow/.completions` 或绝对路径）
    /// * `comment` - 可选的注释文本
    ///
    /// # 错误
    ///
    /// 如果读取或写入配置文件失败，返回相应的错误信息。
    pub fn add_source(source_path: &str, comment: Option<&str>) -> Result<bool> {
        let config_path = Self::get_config_path()?;
        let content = Self::read_config_file(&config_path).unwrap_or_default();

        // 检查是否已存在
        if Self::has_source_in_content(&content, source_path)? {
            return Ok(false);
        }

        // 添加 source 语句
        let mut new_content = content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }

        if let Some(comment_text) = comment {
            new_content.push_str("# ");
            new_content.push_str(comment_text);
            new_content.push('\n');
        }
        new_content.push_str("source ");
        new_content.push_str(source_path);
        new_content.push('\n');
        new_content.push('\n');

        Self::write_config_file(&config_path, &new_content)?;

        Ok(true)
    }

    /// 移除 source 语句
    ///
    /// 从 shell 配置文件中移除指定的 source 语句。
    ///
    /// # 参数
    ///
    /// * `source_path` - source 文件路径
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果移除了 source 语句，否则返回 `false`。
    ///
    /// # 错误
    ///
    /// 如果读取或写入配置文件失败，返回相应的错误信息。
    pub fn remove_source(source_path: &str) -> Result<bool> {
        let config_path = Self::get_config_path()?;
        let content = Self::read_config_file(&config_path).unwrap_or_default();

        // 检查是否存在
        if !Self::has_source_in_content(&content, source_path)? {
            return Ok(false);
        }

        // 移除 source 语句和相关注释
        let new_content = Self::remove_source_from_content(&content, source_path)?;

        Self::write_config_file(&config_path, &new_content)?;

        Ok(true)
    }

    /// 检查 source 语句是否存在
    ///
    /// # 参数
    ///
    /// * `source_path` - source 文件路径
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果 source 语句存在，否则返回 `false`。
    ///
    /// # 错误
    ///
    /// 如果读取配置文件失败，返回相应的错误信息。
    pub fn has_source(source_path: &str) -> Result<bool> {
        let config_path = Self::get_config_path()?;
        let content = Self::read_config_file(&config_path).unwrap_or_default();
        Self::has_source_in_content(&content, source_path)
    }

    /// 获取 shell 的 source 语句关键字
    ///
    /// 不同 shell 使用不同的关键字来加载脚本：
    /// - zsh, bash, fish, elvish: `source`
    /// - powershell: `.`
    fn get_source_keyword(shell: &Shell) -> &'static str {
        match shell {
            Shell::PowerShell => ".",
            _ => "source",
        }
    }

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
        let config_path = Paths::config_file(shell)?;
        let content = Self::read_config_file(&config_path).unwrap_or_default();

        // 检查是否已存在（支持不同格式）
        if Self::has_source_in_content_for_shell(&content, shell, source_path)? {
            return Ok(false);
        }

        // 添加 source 语句
        let mut new_content = content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }

        if let Some(comment_text) = comment {
            new_content.push_str("# ");
            new_content.push_str(comment_text);
            new_content.push('\n');
        }
        let source_keyword = Self::get_source_keyword(shell);
        new_content.push_str(source_keyword);
        new_content.push(' ');
        new_content.push_str(source_path);
        new_content.push('\n');
        new_content.push('\n');

        Self::write_config_file(&config_path, &new_content)?;

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
        let config_path = Paths::config_file(shell)?;
        let content = Self::read_config_file(&config_path).unwrap_or_default();

        // 检查是否存在（支持不同格式）
        if !Self::has_source_in_content_for_shell(&content, shell, source_path)? {
            return Ok(false);
        }

        // 移除 source 语句和相关注释
        let new_content = Self::remove_source_from_content_for_shell(&content, shell, source_path)?;

        Self::write_config_file(&config_path, &new_content)?;

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
        let config_path = Paths::config_file(shell)?;
        let content = Self::read_config_file(&config_path).unwrap_or_default();
        Self::has_source_in_content(&content, source_path)
    }

    // === 配置块管理 ===

    /// 解析配置块
    ///
    /// 从配置内容中解析配置块，返回配置块内的环境变量和移除配置块后的内容。
    fn parse_config_block(content: &str) -> Result<(HashMap<String, String>, String)> {
        let marker_start = "# Workflow CLI Configuration - Start";
        let marker_end = "# Workflow CLI Configuration - End";

        // 提取配置块内的环境变量
        let env_in_block = if let Some(start_pos) = content.find(marker_start) {
            if let Some(end_pos) = content[start_pos..].find(marker_end) {
                let block_content = &content[start_pos + marker_start.len()..start_pos + end_pos];
                Self::parse_shell_config_block(block_content).unwrap_or_default()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // 移除配置块
        let content_without_block = Self::remove_config_block(content, marker_start, marker_end);

        Ok((env_in_block, content_without_block))
    }

    /// 移除配置块
    ///
    /// 从内容中移除配置块（包括标记行）。
    fn remove_config_block(content: &str, marker_start: &str, marker_end: &str) -> String {
        if let Some(start_pos) = content.find(marker_start) {
            if let Some(end_pos) = content[start_pos..].find(marker_end) {
                let end_pos = start_pos + end_pos + marker_end.len();
                let before = content[..start_pos].trim_end();
                let after = content[end_pos..].trim_start();

                if before.is_empty() {
                    after.to_string()
                } else if after.is_empty() {
                    format!("{}\n", before)
                } else {
                    format!("{}\n{}", before, after)
                }
            } else {
                content.to_string()
            }
        } else {
            content.to_string()
        }
    }

    /// 构建配置块
    ///
    /// 根据环境变量构建配置块内容。
    fn build_config_block(env_vars: &HashMap<String, String>) -> String {
        let marker_start = "# Workflow CLI Configuration - Start";
        let marker_end = "# Workflow CLI Configuration - End";

        let mut config_block = String::new();
        config_block.push_str(marker_start);
        config_block.push_str("\n# Generated by Workflow CLI - DO NOT edit manually\n");
        config_block.push_str(
            "# These environment variables will be loaded when you start a new shell\n\n",
        );

        // 按字母顺序排序键
        let mut keys: Vec<&String> = env_vars.keys().collect();
        keys.sort();

        for key in keys {
            let value = &env_vars[key];
            // 转义特殊字符
            let escaped_value = value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('$', "\\$")
                .replace('`', "\\`");
            config_block.push_str("export ");
            config_block.push_str(key);
            config_block.push_str("=\"");
            config_block.push_str(&escaped_value);
            config_block.push_str("\"\n");
        }

        config_block.push('\n');
        config_block.push_str(marker_end);
        config_block.push('\n');

        config_block
    }

    // === 工具方法 ===

    /// 获取 shell 配置文件路径
    ///
    /// 使用 `Detect::shell()` 自动检测 shell 类型，并通过 `Paths::config_file()` 获取对应的配置文件路径。
    ///
    /// # 返回
    ///
    /// 返回 shell 配置文件的路径：
    /// - zsh → `~/.zshrc`
    /// - bash → `~/.bash_profile`（如果不存在则使用 `~/.bashrc`）
    /// - fish → `~/.config/fish/config.fish`
    /// - powershell → `~/.config/powershell/Microsoft.PowerShell_profile.ps1`
    /// - elvish → `~/.elvish/rc.elv`
    ///
    /// # 错误
    ///
    /// 如果无法检测 shell 类型或获取 HOME 目录，返回相应的错误信息。
    pub fn get_config_path() -> Result<PathBuf> {
        let shell = Detect::shell()?;
        Paths::config_file(&shell)
    }

    /// 读取配置文件内容
    fn read_config_file(path: &std::path::Path) -> Result<String> {
        if path.exists() {
            FileReader::new(path).to_string().wrap_err("Failed to read shell config file")
        } else {
            Ok(String::new())
        }
    }

    /// 写入配置文件内容
    fn write_config_file(path: &std::path::Path, content: &str) -> Result<()> {
        FileWriter::new(path)
            .write_str(content)
            .wrap_err("Failed to write to shell config file")?;
        Ok(())
    }

    /// 加载现有配置
    fn load_existing_config(path: &std::path::Path) -> Result<ExistingConfig> {
        let content = Self::read_config_file(path)?;
        let (env_in_block, content_without_block) = Self::parse_config_block(&content)?;

        Ok(ExistingConfig {
            env_in_block,
            content_without_block,
        })
    }

    /// 解析 shell 配置块中的 export KEY="VALUE" 格式
    fn parse_shell_config_block(block_content: &str) -> Result<HashMap<String, String>> {
        let mut env_vars = HashMap::new();

        for line in block_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // 匹配 export KEY="VALUE" 或 export KEY=VALUE
            if let Some(rest) = line.strip_prefix("export ") {
                if let Some(equal_pos) = rest.find('=') {
                    let key = rest[..equal_pos].trim();
                    let mut value = rest[equal_pos + 1..].trim();
                    // 移除引号（如果有）
                    if (value.starts_with('"') && value.ends_with('"'))
                        || (value.starts_with('\'') && value.ends_with('\''))
                    {
                        value = &value[1..value.len() - 1];
                    }
                    // 反转义
                    let unescaped_value = value
                        .replace("\\\\", "\\")
                        .replace("\\\"", "\"")
                        .replace("\\$", "$")
                        .replace("\\`", "`");
                    if !key.is_empty() {
                        env_vars.insert(key.to_string(), unescaped_value);
                    }
                }
            }
        }

        Ok(env_vars)
    }

    /// 合并环境变量
    fn merge_env_vars(
        existing: &HashMap<String, String>,
        new_vars: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut merged = existing.clone();
        merged.extend(new_vars.iter().map(|(k, v)| (k.clone(), v.clone())));
        merged
    }

    /// 构建配置内容
    fn build_config_content(
        content_without_block: &str,
        env_vars: &HashMap<String, String>,
    ) -> Result<String> {
        // 如果没有环境变量，不创建配置块，直接返回原内容
        if env_vars.is_empty() {
            return Ok(content_without_block.to_string());
        }

        let config_block = Self::build_config_block(env_vars);

        // 合并内容
        let final_content = if content_without_block.is_empty() {
            config_block
        } else if content_without_block.ends_with('\n') {
            format!("{}{}", content_without_block, config_block)
        } else {
            format!("{}\n{}", content_without_block, config_block)
        };

        Ok(final_content)
    }

    /// 检查内容中是否包含 source 语句
    fn has_source_in_content(content: &str, source_path: &str) -> Result<bool> {
        // 检查相对路径模式
        if content.contains(source_path) {
            return Ok(true);
        }

        // 检查绝对路径（如果 source_path 是相对路径）
        if source_path.contains("$HOME") {
            let home = Paths::home_dir()?;
            let abs_path = source_path.replace("$HOME", &home.to_string_lossy());
            if content.contains(&abs_path) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 检查内容中是否包含 source 语句（指定 shell 类型）
    ///
    /// 支持不同 shell 的 source 语句格式（PowerShell 使用 `.`，其他使用 `source`）。
    fn has_source_in_content_for_shell(
        content: &str,
        shell: &Shell,
        source_path: &str,
    ) -> Result<bool> {
        let source_keyword = Self::get_source_keyword(shell);

        // 检查 source 语句（支持不同关键字）
        let patterns = vec![
            format!("{} {}", source_keyword, source_path),
            format!("{}  {}", source_keyword, source_path), // 支持多个空格
        ];

        for pattern in &patterns {
            if content.contains(pattern) {
                return Ok(true);
            }
        }

        // 检查绝对路径（如果 source_path 是相对路径）
        if source_path.contains("$HOME") {
            let home = Paths::home_dir()?;
            let abs_path = source_path.replace("$HOME", &home.to_string_lossy());
            let abs_patterns = vec![
                format!("{} {}", source_keyword, abs_path),
                format!("{}  {}", source_keyword, abs_path),
            ];

            for pattern in &abs_patterns {
                if content.contains(pattern) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// 从内容中移除 source 语句
    ///
    /// 移除 source 语句及其相关的注释块（如果存在）。
    fn remove_source_from_content(content: &str, source_path: &str) -> Result<String> {
        let home = Paths::home_dir()?;
        let abs_path = source_path.replace("$HOME", &home.to_string_lossy());

        let mut new_content = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // 检查是否是配置块开始（包含 Workflow CLI 的注释）
            if line.contains("# Workflow CLI")
                && (line.contains("completions") || line.contains("Configuration"))
            {
                // 跳过整个配置块
                i += 1; // 跳过注释行
                        // 跳过 source 行
                while i < lines.len() {
                    let current_line = lines[i];
                    if current_line.contains(source_path) || current_line.contains(&abs_path) {
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
            if line.contains(source_path) || line.contains(&abs_path) {
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

        Ok(new_content)
    }

    /// 从内容中移除 source 语句（指定 shell 类型）
    ///
    /// 移除 source 语句及其相关的注释块（如果存在）。
    /// 支持不同 shell 的 source 语句格式（PowerShell 使用 `.`，其他使用 `source`）。
    fn remove_source_from_content_for_shell(
        content: &str,
        shell: &Shell,
        source_path: &str,
    ) -> Result<String> {
        let home = Paths::home_dir()?;
        let abs_path = source_path.replace("$HOME", &home.to_string_lossy());
        let source_keyword = Self::get_source_keyword(shell);

        let mut new_content = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // 检查是否是配置块开始（包含 Workflow CLI 的注释）
            if line.contains("# Workflow CLI")
                && (line.contains("completions") || line.contains("Configuration"))
            {
                // 跳过整个配置块
                i += 1; // 跳过注释行
                        // 跳过 source 行
                while i < lines.len() {
                    let current_line = lines[i];
                    // 检查是否匹配 source 语句（支持不同关键字和路径格式）
                    let matches_source = (current_line.trim().starts_with(source_keyword)
                        && (current_line.contains(source_path)
                            || current_line.contains(&abs_path)))
                        || (current_line.contains(source_path) || current_line.contains(&abs_path));

                    if matches_source {
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
            let matches_source = (line.trim().starts_with(source_keyword)
                && (line.contains(source_path) || line.contains(&abs_path)))
                || (line.contains(source_path) || line.contains(&abs_path));

            if matches_source {
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

        Ok(new_content)
    }
}

/// 现有配置
struct ExistingConfig {
    env_in_block: HashMap<String, String>,
    content_without_block: String,
}
