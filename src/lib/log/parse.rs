//! 日志解析模块
//! 解析日志条目，提取基本信息

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

use super::extract::extract_url_from_line;

/// 日志条目信息
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: Option<String>,
    pub url: Option<String>,
}

/// 添加条目到结果列表（如果未重复）
pub fn add_entry_if_not_duplicate(
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

/// 解析日志条目（从以 💡 开头的行）
pub fn parse_log_entry(line: &str) -> Result<Option<LogEntry>> {
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
