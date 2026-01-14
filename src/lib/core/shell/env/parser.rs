//! 环境变量解析器
//!
//! 提供从 shell 配置文件中解析环境变量的功能。

use color_eyre::Result;
use std::collections::HashMap;

use crate::core::shell::block;

/// 解析配置块
///
/// 从配置内容中解析配置块，返回配置块内的环境变量和移除配置块后的内容。
pub fn parse_config_block(content: &str) -> Result<(HashMap<String, String>, String)> {
    // 提取配置块内的环境变量
    let block_content = block::extract_content(content);
    let env_in_block = if block_content.is_empty() {
        HashMap::new()
    } else {
        parse_shell_config_block(&block_content).unwrap_or_default()
    };

    // 移除配置块
    let content_without_block = block::remove(content);

    Ok((env_in_block, content_without_block))
}

/// 解析 shell 配置块中的 export KEY="VALUE" 格式
pub fn parse_shell_config_block(block_content: &str) -> Result<HashMap<String, String>> {
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
