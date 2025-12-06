//! 日志处理辅助函数
//!
//! 本模块提供了日志处理相关的辅助函数，包括：
//! - 日志条目解析和 URL 提取
//! - 路径处理
//! - 文件操作
//! - 目录信息计算

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use walkdir::WalkDir;

use crate::base::settings::paths::Paths;

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

/// 展开路径字符串
///
/// 支持的路径格式：
/// - Unix: `~` 和 `~/path` - 展开为用户主目录
/// - Windows: `%VAR%` 和 `%VAR%\path` - 展开环境变量
/// - 绝对路径: 直接使用
///
/// # 示例
///
/// ```
/// // Unix
/// expand_path("~/Documents/Workflow") -> "/home/user/Documents/Workflow"
/// expand_path("~") -> "/home/user"
///
/// // Windows
/// expand_path("%USERPROFILE%\\Documents\\Workflow") -> "C:\\Users\\User\\Documents\\Workflow"
/// expand_path("%APPDATA%\\workflow") -> "C:\\Users\\User\\AppData\\Roaming\\workflow"
///
/// // 绝对路径
/// expand_path("/absolute/path") -> "/absolute/path"
/// expand_path("C:\\absolute\\path") -> "C:\\absolute\\path"
/// ```
pub(crate) fn expand_path(path_str: &str) -> Result<PathBuf> {
    // 处理 Unix 风格的 ~ 展开
    if let Some(rest) = path_str.strip_prefix("~/") {
        // 使用统一的 home_dir 方法
        let home = Paths::home_dir()?;
        return Ok(home.join(rest));
    }
    if path_str == "~" {
        // 使用统一的 home_dir 方法
        return Paths::home_dir();
    }

    // 处理 Windows 风格的环境变量展开 %VAR%
    if path_str.contains('%') {
        let mut result = String::new();
        let mut chars = path_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                // 提取环境变量名
                let mut var_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '%' {
                        chars.next(); // 跳过结束的 %
                        break;
                    }
                    var_name.push(chars.next().unwrap());
                }

                // 展开环境变量
                if !var_name.is_empty() {
                    let var_value = env::var(&var_name)
                        .with_context(|| format!("Environment variable not set: {}", var_name))?;
                    result.push_str(&var_value);
                }
            } else {
                result.push(ch);
            }
        }

        return Ok(PathBuf::from(result));
    }

    // 其他情况：直接使用路径（可能是绝对路径或相对路径）
    Ok(PathBuf::from(path_str))
}

/// 打开日志文件并返回 BufReader
pub(crate) fn open_log_file(log_file: &Path) -> Result<BufReader<File>> {
    let file =
        File::open(log_file).with_context(|| format!("Failed to open log file: {:?}", log_file))?;
    Ok(BufReader::new(file))
}

/// 格式化文件大小
pub(crate) fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 计算目录大小和文件数量
pub(crate) fn calculate_dir_info(dir: &Path) -> Result<(u64, usize)> {
    let mut total_size = 0u64;
    let mut file_count = 0usize;

    if !dir.exists() {
        return Ok((0, 0));
    }

    for entry in WalkDir::new(dir) {
        let entry = entry.context("Failed to read directory entry")?;
        let metadata = entry.metadata().context("Failed to get file metadata")?;

        if metadata.is_file() {
            total_size += metadata.len();
            file_count += 1;
        }
    }

    Ok((total_size, file_count))
}

/// 列出目录内容
pub(crate) fn list_dir_contents(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut contents = Vec::new();

    if !dir.exists() {
        return Ok(contents);
    }

    for entry in WalkDir::new(dir).max_depth(3) {
        let entry = entry.context("Failed to read directory entry")?;
        contents.push(entry.path().to_path_buf());
    }

    Ok(contents)
}
