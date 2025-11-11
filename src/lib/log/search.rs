//! 搜索模块
//! 在日志文件中搜索关键词

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use std::io::BufRead;
use std::path::Path;
use std::sync::OnceLock;

use super::extract::extract_url_from_line;
use super::parse::{add_entry_if_not_duplicate, parse_log_entry, LogEntry};
use super::utils::open_log_file;

/// 在日志文件中搜索关键词
/// 返回匹配的请求信息列表（URL 和 ID）
///
/// 支持两种日志格式：
/// 1. flutter-api.log 格式：以 💡 开头的行
/// 2. api.log 格式：包含 `#<数字> <HTTP方法> <URL>` 的行
///
/// 匹配 shell 脚本 qksearch.sh 的逻辑：
/// 1. 查找新日志条目（💡 开头或包含 `#<数字> <HTTP方法>` 的行）
/// 2. 提取 ID（#<数字>）和 URL
/// 3. 在当前块中搜索关键词（不区分大小写），包括条目行本身
/// 4. 如果找到匹配，记录该块的 URL 和 ID
/// 5. 空行表示块结束
pub fn search_keyword(log_file: &Path, keyword: &str) -> Result<Vec<LogEntry>> {
    let reader = open_log_file(log_file)?;
    let keyword_lower = keyword.to_lowercase();
    let mut results = Vec::new();
    let mut printed_ids = HashSet::new();
    let mut current_entry: Option<LogEntry> = None;
    let mut found_in_current_block = false;

    // 用于检测 api.log 格式的条目（包含 `#<数字> <HTTP方法>` 的模式）
    // 使用静态正则表达式避免重复编译
    static API_LOG_ENTRY_PATTERN: OnceLock<Option<Regex>> = OnceLock::new();
    let api_log_entry_pattern = API_LOG_ENTRY_PATTERN
        .get_or_init(|| Regex::new(r"#\d+\s+(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)").ok());

    for line_result in reader.lines() {
        let line = line_result.context("Failed to read line")?;
        let line_lower = line.to_lowercase();

        // 检查是否是新条目的开始
        let is_new_entry = if line.starts_with("💡") {
            // flutter-api.log 格式：以 💡 开头
            true
        } else if let Some(pattern) = api_log_entry_pattern.as_ref() {
            // api.log 格式：包含 `#<数字> <HTTP方法>` 的模式
            pattern.is_match(&line)
        } else {
            false
        };

        if is_new_entry {
            // 如果之前的条目匹配，保存它（避免重复）
            if found_in_current_block {
                add_entry_if_not_duplicate(current_entry.take(), &mut results, &mut printed_ids);
            }

            // 解析新条目
            current_entry = parse_log_entry(&line)?;
            // 在条目行本身也搜索关键词（因为 URL 通常在这一行）
            found_in_current_block = line_lower.contains(&keyword_lower);
        } else if current_entry.is_some() {
            // 在当前块中搜索关键词（不区分大小写）
            if line_lower.contains(&keyword_lower) {
                found_in_current_block = true;
            }

            // 提取 URL（如果需要）
            if let Some(ref mut entry) = current_entry {
                if entry.url.is_none() {
                    entry.url = extract_url_from_line(&line);
                }
            }
        }

        // 空行表示块结束
        if line.trim().is_empty() {
            // 如果当前块匹配，保存结果
            if found_in_current_block {
                add_entry_if_not_duplicate(current_entry.take(), &mut results, &mut printed_ids);
            }
            // 重置状态
            current_entry = None;
            found_in_current_block = false;
        }
    }

    // 检查最后一个条目
    if found_in_current_block {
        add_entry_if_not_duplicate(current_entry, &mut results, &mut printed_ids);
    }

    Ok(results)
}
