//! 搜索和查找功能相关实现

use color_eyre::{eyre::WrapErr, Result};
use regex::Regex;
use std::collections::HashSet;
use std::io::BufRead;
use std::path::Path;
use std::sync::OnceLock;

use crate::base::fs::FileReader;

use super::constants::*;
use super::helpers;
use super::helpers::LogEntry;
use super::JiraLogs;

impl JiraLogs {
    /// 从日志文件中搜索请求 ID
    ///
    /// 自动根据 `jira_id` 解析日志文件路径，然后查找请求 ID。
    pub fn find_request_id(&self, jira_id: &str, request_id: &str) -> Result<Option<LogEntry>> {
        let log_file = self.ensure_log_file_exists(jira_id)?;

        let reader = FileReader::new(&log_file).open()?;
        let mut current_entry: Option<LogEntry> = None;
        let mut found_id = false;

        for line_result in reader.lines() {
            let line = line_result.wrap_err("Failed to read line")?;

            // 检查是否包含请求 ID（匹配 shell 脚本：$0 ~ "#" rid）
            if line.contains(&format!("#{}", request_id)) {
                // 解析条目（提取 ID 和 URL）
                current_entry = helpers::parse_log_entry(&line)?;

                // 验证 ID 是否匹配
                if let Some(ref entry) = current_entry {
                    if entry.id.as_ref().map(|id| id == request_id).unwrap_or(false) {
                        found_id = true;
                        // 如果 URL 还没有提取，尝试从当前行提取
                        if entry.url.is_none() {
                            if let Some(ref mut entry) = current_entry {
                                entry.url = helpers::extract_url_from_line(&line);
                            }
                        }
                        break; // 找到后立即返回
                    }
                }
            }
        }

        if found_id {
            Ok(current_entry)
        } else {
            Ok(None)
        }
    }

    /// 提取日志条目的响应内容
    ///
    /// 自动根据 `jira_id` 解析日志文件路径，然后提取响应内容。
    pub fn extract_response_content(&self, jira_id: &str, request_id: &str) -> Result<String> {
        let log_file = self.ensure_log_file_exists(jira_id)?;

        let reader = FileReader::new(&log_file).open()?;
        let mut lines = reader.lines();
        let mut response_lines = Vec::new();
        let mut prev_line = String::new();
        let mut in_response = false;

        while let Some(Ok(line)) = lines.next() {
            // 检查是否包含 response:
            if line.contains(RESPONSE_KEYWORD) {
                // 检查上一行是否包含请求 ID
                if prev_line.contains(&format!("#{}", request_id)) {
                    in_response = true;
                    // 提取 "response: " 之后的内容
                    if let Some(response_start) = line.find(RESPONSE_KEYWORD) {
                        let response_content =
                            &line[response_start + RESPONSE_KEYWORD_LEN..].trim_start();
                        if !response_content.is_empty() {
                            response_lines.push(response_content.to_string());
                        }
                    }
                    continue;
                }
            }

            // 如果在响应块中，收集内容直到空行
            if in_response {
                if line.trim().is_empty() {
                    break; // 空行表示响应结束
                }
                response_lines.push(line.clone());
            }

            // 保存当前行作为下一行的 prev
            prev_line = line.clone();
        }

        Ok(response_lines.join("\n"))
    }

    /// 在指定日志文件中搜索关键词（内部方法）
    fn search_keyword_in_file(&self, log_file: &Path, keyword: &str) -> Result<Vec<LogEntry>> {
        // 如果文件不存在，返回空结果
        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let reader = FileReader::new(log_file).open()?;
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();
        let mut printed_ids = HashSet::new();
        let mut current_entry: Option<LogEntry> = None;
        let mut found_in_current_block = false;

        for line_result in reader.lines() {
            let line = line_result.wrap_err("Failed to read line")?;
            let line_lower = line.to_lowercase();

            // 检查是否是新条目的开始
            if self.is_new_log_entry(&line) {
                // 如果之前的条目匹配，保存它（避免重复）
                if found_in_current_block {
                    helpers::add_entry_if_not_duplicate(
                        current_entry.take(),
                        &mut results,
                        &mut printed_ids,
                    );
                }

                // 解析新条目
                current_entry = helpers::parse_log_entry(&line)?;
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
                        entry.url = helpers::extract_url_from_line(&line);
                    }
                }
            }

            // 空行表示块结束
            if line.trim().is_empty() {
                // 如果当前块匹配，保存结果
                if found_in_current_block {
                    helpers::add_entry_if_not_duplicate(
                        current_entry.take(),
                        &mut results,
                        &mut printed_ids,
                    );
                }
                // 重置状态
                current_entry = None;
                found_in_current_block = false;
            }
        }

        // 检查最后一个条目
        if found_in_current_block {
            helpers::add_entry_if_not_duplicate(current_entry, &mut results, &mut printed_ids);
        }

        Ok(results)
    }

    /// 在日志文件中搜索关键词
    ///
    /// 自动根据 `jira_id` 解析日志文件路径，然后搜索关键词。
    /// 同时搜索 flutter-api.log 和 api.log 两个文件。
    pub fn search_keyword(&self, jira_id: &str, keyword: &str) -> Result<Vec<LogEntry>> {
        let log_file = self.ensure_log_file_exists(jira_id)?;
        self.search_keyword_in_file(&log_file, keyword)
    }

    /// 同时搜索 flutter-api.log 和 api.log 文件
    ///
    /// 返回两个文件的结果，分别对应 (api.log 结果, flutter-api.log 结果)
    pub fn search_keyword_both_files(
        &self,
        jira_id: &str,
        keyword: &str,
    ) -> Result<(Vec<LogEntry>, Vec<LogEntry>)> {
        // 搜索 flutter-api.log
        let flutter_api_log = self.ensure_log_file_exists(jira_id)?;
        let flutter_api_results = self.search_keyword_in_file(&flutter_api_log, keyword)?;

        // 搜索 api.log（如果存在）
        let api_log = self.get_api_log_file_path(jira_id)?;
        let api_results = self.search_keyword_in_file(&api_log, keyword)?;

        Ok((api_results, flutter_api_results))
    }

    /// 检测是否是新日志条目的开始
    fn is_new_log_entry(&self, line: &str) -> bool {
        // flutter-api.log 格式：以 💡 开头
        if line.starts_with("💡") {
            return true;
        }

        // api.log 格式：包含 `#<数字> <HTTP方法>` 的模式
        // 使用静态正则表达式避免重复编译
        static API_LOG_ENTRY_PATTERN: OnceLock<Option<Regex>> = OnceLock::new();
        let api_log_entry_pattern = API_LOG_ENTRY_PATTERN
            .get_or_init(|| Regex::new(r"#\d+\s+(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)").ok());

        if let Some(pattern) = api_log_entry_pattern.as_ref() {
            pattern.is_match(line)
        } else {
            false
        }
    }
}
