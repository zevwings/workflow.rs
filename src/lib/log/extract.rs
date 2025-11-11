//! URL 提取模块
//! 从日志行中提取 URL

use regex::Regex;
use std::sync::OnceLock;

/// 从行中提取 URL
///
/// 匹配 shell 脚本的逻辑：
/// 1. 首先尝试匹配 HTTP 方法（GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS）后的 URL
/// 2. 如果没有找到，尝试匹配格式：`数字 https://...`
/// 3. 清理 URL（移除引号、单引号、空格、逗号、右花括号等）
pub fn extract_url_from_line(line: &str) -> Option<String> {
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
