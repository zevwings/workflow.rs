//! Shell 环境变量管理
//!
//! 提供 shell 配置文件中的环境变量管理功能，包括：
//! - 从配置块加载环境变量
//! - 保存环境变量到配置块
//! - 移除环境变量

mod builder;
mod parser;

use color_eyre::Result;
use std::collections::HashMap;

use crate::core::shell::block;
use crate::core::shell::file;
use crate::core::shell::paths;

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
    let config_path = paths::get_config_path()?;

    if !config_path.exists() {
        return Ok(HashMap::new());
    }

    let content = file::read_config_file(&config_path)?;
    let (env_vars, _) = parser::parse_config_block(&content)?;

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
    let config_path = paths::get_config_path()?;

    // 读取现有配置
    let existing = load_existing_config(&config_path)?;

    // 合并环境变量
    let merged = builder::merge_env_vars(&existing.env_in_block, env_vars);

    // 构建新内容
    let new_content = builder::build_config_content(&existing.content_without_block, &merged)?;

    // 写入文件
    file::write_config_file(&config_path, &new_content)?;

    Ok(())
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
    let config_path = paths::get_config_path()?;

    if !config_path.exists() {
        return Ok(false);
    }

    let content = file::read_config_file(&config_path)?;

    // 使用 filter 直接过滤掉需要删除的行，避免先收集再删除
    let lines: Vec<&str> = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();

            // 跳过配置块的标记行
            if trimmed == block::MARKER_START || trimmed == block::MARKER_END {
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
        let final_content = remove_empty_config_block(&new_content);

        // 确保文件以换行符结尾
        let final_content = if final_content.ends_with('\n') {
            final_content
        } else {
            // 使用 with_capacity 优化
            let mut result = String::with_capacity(final_content.len() + 1);
            result.push_str(&final_content);
            result.push('\n');
            result
        };

        file::write_config_file(&config_path, &final_content)?;
    }

    Ok(removed_any)
}

// === 私有辅助函数 ===

/// 现有配置
struct ExistingConfig {
    env_in_block: HashMap<String, String>,
    content_without_block: String,
}

/// 加载现有配置
fn load_existing_config(path: &std::path::Path) -> Result<ExistingConfig> {
    let content = file::read_config_file(path)?;
    let (env_in_block, content_without_block) = parser::parse_config_block(&content)?;

    Ok(ExistingConfig {
        env_in_block,
        content_without_block,
    })
}

/// 移除空的配置块
///
/// 如果配置块内没有任何 export 语句（只有标记和注释），则完全移除配置块。
fn remove_empty_config_block(content: &str) -> String {
    // 提取配置块内容
    let block_content = block::extract_content(content);

    // 检查块内是否有任何 export 语句
    let has_exports = !block::is_empty(&block_content, |line| line.starts_with("export "));

    // 如果没有 export 语句，移除整个配置块
    if !has_exports {
        block::remove(content)
    } else {
        content.to_string()
    }
}
