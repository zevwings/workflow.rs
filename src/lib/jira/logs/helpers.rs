//! 日志处理辅助函数
//!
//! 本模块提供了日志处理相关的辅助函数，包括：
//! - 日志条目解析和 URL 提取
//! - 路径处理
//! - 文件操作

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

/// 日志条目信息
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: Option<String>,
    pub url: Option<String>,
}

/// 从行中提取 URL
///
/// 匹配 shell 脚本的逻辑：
/// 1. 首先尝试匹配 HTTP 方法（GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS）后的 URL
/// 2. 如果没有找到，尝试匹配格式：`数字 https://...`
/// 3. 清理 URL（移除引号、单引号、空格、逗号、右花括号等）
pub(crate) fn extract_url_from_line(line: &str) -> Option<String> {
    // 清理 URL 的辅助函数
    fn clean_url(url: &str) -> String {
        url.trim_end_matches(['"', '\'', ' ', ',', '}']).to_string()
    }

    // 方法 1: 查找 HTTP 方法后的 URL
    // 匹配: GET https://... 或 POST https://... 等
    // 使用静态正则表达式避免重复编译
    static METHOD_PATTERN: OnceLock<Regex> = OnceLock::new();
    let method_pattern = METHOD_PATTERN.get_or_init(|| {
        Regex::new("(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\\s+(https?://[^\\s\",]+)")
            .expect("Failed to compile method pattern regex")
    });

    if let Some(caps) = method_pattern.captures(line) {
        if let Some(url_match) = caps.get(2) {
            return Some(clean_url(url_match.as_str()));
        }
    }

    // 方法 2: 查找格式 `数字 https://...`
    // 匹配: 200 https://... 或 404 https://... 等
    // 使用静态正则表达式避免重复编译
    static STATUS_PATTERN: OnceLock<Regex> = OnceLock::new();
    let status_pattern = STATUS_PATTERN.get_or_init(|| {
        Regex::new(r#"\d+\s+(https?://[^\s",]+)"#).expect("Failed to compile status pattern regex")
    });

    if let Some(caps) = status_pattern.captures(line) {
        if let Some(url_match) = caps.get(1) {
            return Some(clean_url(url_match.as_str()));
        }
    }

    None
}

/// 解析日志条目（从以 💡 开头的行）
pub(crate) fn parse_log_entry(line: &str) -> Result<Option<LogEntry>> {
    // 提取 ID（#123 格式）- 使用静态正则表达式避免重复编译
    static ID_REGEX: OnceLock<Regex> = OnceLock::new();
    let id_re = ID_REGEX.get_or_init(|| Regex::new(r"#(\d+)").expect("Failed to compile ID regex"));
    let id = id_re
        .captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string());

    // 尝试提取 URL
    let url = extract_url_from_line(line);

    Ok(Some(LogEntry { id, url }))
}

/// 添加条目到结果列表（如果未重复）
pub(crate) fn add_entry_if_not_duplicate(
    entry: Option<LogEntry>,
    results: &mut Vec<LogEntry>,
    printed_ids: &mut HashSet<String>,
) {
    if let Some(entry) = entry {
        if let Some(ref id) = entry.id {
            if !printed_ids.contains(id) {
                let id_clone = id.clone();
                results.push(entry);
                printed_ids.insert(id_clone);
            }
        }
    }
}
