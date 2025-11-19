//! 代理环境变量管理
//!
//! 本模块提供了代理环境变量的管理功能，包括：
//! - 从 shell 配置文件读取代理环境变量（~/.zshrc, ~/.bash_profile 等）
//! - 保存代理环境变量到 shell 配置文件
//! - 删除代理环境变量
//!
//! 代理环境变量保存在 shell 配置文件的配置块中，格式：
//! ```
//! # Workflow CLI Configuration - Start
//! export KEY="VALUE"
//! # Workflow CLI Configuration - End
//! ```

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 环境变量管理器（仅用于代理功能）
///
/// 管理代理相关的环境变量（http_proxy, https_proxy, all_proxy）。
/// 数据源：shell 配置文件（~/.zshrc, ~/.bash_profile 等）。
pub struct EnvFile;

impl EnvFile {
    /// 从 shell 配置文件加载环境变量
    ///
    /// 从 shell 配置文件中读取配置块内的环境变量（主要用于代理配置）。
    ///
    /// # 返回
    ///
    /// 返回环境变量 HashMap。如果配置文件不存在或没有配置块，返回空 HashMap。
    ///
    /// # 错误
    ///
    /// 如果读取配置文件失败，返回相应的错误信息。
    pub fn load() -> Result<HashMap<String, String>> {
        let shell_config_path = Self::get_shell_config_path()?;

        if !shell_config_path.exists() {
            return Ok(HashMap::new());
        }

        let content =
            fs::read_to_string(&shell_config_path).context("Failed to read shell config file")?;

        // 解析配置块中的环境变量
        let marker_start = "# Workflow CLI Configuration - Start";
        let marker_end = "# Workflow CLI Configuration - End";

        if let Some(start_pos) = content.find(marker_start) {
            if let Some(end_pos) = content[start_pos..].find(marker_end) {
                let block_content = &content[start_pos + marker_start.len()..start_pos + end_pos];
                return Self::parse_shell_config_block(block_content);
            }
        }

        // 如果找不到配置块，返回空
        Ok(HashMap::new())
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

    /// 保存环境变量到 shell 配置文件
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
    pub fn save(env_vars: &HashMap<String, String>) -> Result<()> {
        // 保存到 shell 配置文件（用户主目录，使其在 shell 中生效）
        Self::save_to_shell_config(env_vars)?;
        Ok(())
    }

    /// 保存环境变量到 shell 配置文件（用户主目录）
    /// 如果 key 已存在则覆盖，不存在则新增
    fn save_to_shell_config(env_vars: &HashMap<String, String>) -> Result<()> {
        let shell_config_path = Self::get_shell_config_path()?;

        // 读取现有配置文件内容
        let existing_content = if shell_config_path.exists() {
            fs::read_to_string(&shell_config_path).context("Failed to read shell config file")?
        } else {
            String::new()
        };

        // 确定标记行的位置（用于识别 Workflow CLI 配置块）
        let marker_start = "# Workflow CLI Configuration - Start";
        let marker_end = "# Workflow CLI Configuration - End";

        // 解析现有配置块中的环境变量（如果存在）
        let existing_env_in_block = if let Some(start_pos) = existing_content.find(marker_start) {
            if let Some(end_pos) = existing_content[start_pos..].find(marker_end) {
                let block_content =
                    &existing_content[start_pos + marker_start.len()..start_pos + end_pos];
                Self::parse_shell_config_block(block_content).unwrap_or_default()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // 合并环境变量：新值覆盖旧值
        let mut merged_env = existing_env_in_block;
        for (key, value) in env_vars {
            merged_env.insert(key.clone(), value.clone());
        }

        // 移除旧的配置块
        let mut content_without_block = existing_content.clone();
        if let Some(start_pos) = content_without_block.find(marker_start) {
            if let Some(end_pos) = content_without_block[start_pos..].find(marker_end) {
                let end_pos = start_pos + end_pos + marker_end.len();
                let before = content_without_block[..start_pos].trim_end();
                let after = content_without_block[end_pos..].trim_start();
                content_without_block = if before.is_empty() {
                    after.to_string()
                } else if after.is_empty() {
                    format!("{}\n", before)
                } else {
                    format!("{}\n{}", before, after)
                };
            }
        }

        // 构建新的配置块
        let mut config_block = String::new();
        config_block.push_str(&format!("{}\n", marker_start));
        config_block.push_str("# Generated by Workflow CLI - DO NOT edit manually\n");
        config_block.push_str(
            "# These environment variables will be loaded when you start a new shell\n\n",
        );

        // 按字母顺序排序键
        let mut keys: Vec<&String> = merged_env.keys().collect();
        keys.sort();

        // 写入配置项（使用 export 格式）
        for key in keys {
            let value = merged_env.get(key).unwrap();
            // 转义特殊字符
            let escaped_value = value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('$', "\\$")
                .replace('`', "\\`");

            config_block.push_str(&format!("export {}=\"{}\"\n", key, escaped_value));
        }

        config_block.push_str(&format!("{}\n", marker_end));

        // 将更新后的内容和新的配置块组合
        let mut final_content = content_without_block;
        if !final_content.is_empty() && !final_content.ends_with('\n') {
            final_content.push('\n');
        }
        final_content.push_str(&config_block);

        // 写入整个文件
        fs::write(&shell_config_path, final_content)
            .context("Failed to write to shell config file")?;

        Ok(())
    }

    /// 根据用户的 SHELL 环境变量确定 shell 配置文件路径
    ///
    /// 根据 `SHELL` 环境变量自动检测 shell 类型，并返回对应的配置文件路径。
    ///
    /// # 返回
    ///
    /// 返回 shell 配置文件的路径：
    /// - zsh → `~/.zshrc`
    /// - bash → `~/.bash_profile`（如果不存在则使用 `~/.bashrc`）
    /// - 其他 → `~/.profile`
    ///
    /// # 错误
    ///
    /// 如果 `HOME` 环境变量未设置，返回相应的错误信息。
    pub fn get_shell_config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let home_dir = PathBuf::from(home);

        // 获取用户的 SHELL 环境变量
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        // 根据 shell 类型确定配置文件路径
        let config_path = if shell.contains("zsh") {
            home_dir.join(".zshrc")
        } else if shell.contains("bash") {
            // macOS 通常使用 .bash_profile，Linux 使用 .bashrc
            // 这里优先使用 .bash_profile，如果不存在则使用 .bashrc
            let bash_profile = home_dir.join(".bash_profile");
            let bashrc = home_dir.join(".bashrc");

            // 如果 .bash_profile 不存在但 .bashrc 存在，使用 .bashrc
            // 否则使用 .bash_profile
            if !bash_profile.exists() && bashrc.exists() {
                bashrc
            } else {
                bash_profile
            }
        } else if shell.contains("csh") && !shell.contains("tcsh") {
            home_dir.join(".cshrc")
        } else if shell.contains("tcsh") {
            home_dir.join(".tcshrc")
        } else if shell.contains("ksh") {
            home_dir.join(".kshrc")
        } else {
            // 默认使用 .profile
            home_dir.join(".profile")
        };

        Ok(config_path)
    }

    /// 批量设置环境变量
    pub fn set_multiple(env_vars: &HashMap<String, String>) -> Result<()> {
        Self::save(env_vars)
    }

    /// 从整个文件中移除指定的环境变量（包括配置块内外）
    pub fn remove_from_file(keys: &[&str]) -> Result<()> {
        let shell_config_path = Self::get_shell_config_path()?;

        if !shell_config_path.exists() {
            return Ok(());
        }

        let content =
            fs::read_to_string(&shell_config_path).context("Failed to read shell config file")?;

        let marker_start = "# Workflow CLI Configuration - Start";
        let marker_end = "# Workflow CLI Configuration - End";

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut removed_any = false;

        // 收集需要删除的行索引
        let mut indices_to_remove: Vec<usize> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // 跳过配置块的标记行
            if trimmed == marker_start || trimmed == marker_end {
                continue;
            }

            // 如果行以 export 开头，检查是否匹配要删除的键
            if let Some(rest) = trimmed.strip_prefix("export ") {
                for key in keys {
                    // 检查是否是 export KEY= 或 export KEY="
                    if let Some(after_key) = rest.strip_prefix(key) {
                        let after_key = after_key.trim_start();
                        if after_key.starts_with('=') {
                            // 找到匹配的 export 行
                            indices_to_remove.push(i);
                            removed_any = true;
                            break;
                        }
                    }
                }
            }
        }

        // 从后往前删除
        indices_to_remove.sort_by(|a, b| b.cmp(a));
        for idx in indices_to_remove {
            lines.remove(idx);
        }

        if removed_any {
            // 重新构建文件内容
            let new_content = lines.join("\n");

            // 确保文件以换行符结尾
            let final_content = if new_content.ends_with('\n') {
                new_content
            } else {
                format!("{}\n", new_content)
            };

            fs::write(&shell_config_path, final_content)
                .context("Failed to write to shell config file")?;
        }

        Ok(())
    }
}
