//! 查找模块
//! 查找请求 ID 并提取响应内容

use anyhow::{Context, Result};
use std::env;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use crate::Settings;

use super::extract::extract_url_from_line;
use super::parse::{parse_log_entry, LogEntry};
use super::utils::{expand_path, open_log_file};

/// 从日志文件中搜索请求 ID
/// 返回包含该 ID 的条目信息
///
/// 匹配 shell 脚本 qkfind.sh 的逻辑：
/// 1. 查找包含 `#<request_id>` 的行
/// 2. 从该行中提取 URL（如果存在）
/// 3. 返回包含 ID 和 URL 的条目
pub fn find_request_id(log_file: &Path, request_id: &str) -> Result<Option<LogEntry>> {
    let reader = open_log_file(log_file)?;
    let mut current_entry: Option<LogEntry> = None;
    let mut found_id = false;

    for line_result in reader.lines() {
        let line = line_result.context("Failed to read line")?;

        // 检查是否包含请求 ID（匹配 shell 脚本：$0 ~ "#" rid）
        if line.contains(&format!("#{}", request_id)) {
            // 解析条目（提取 ID 和 URL）
            current_entry = parse_log_entry(&line)?;

            // 验证 ID 是否匹配
            if let Some(ref entry) = current_entry {
                if entry
                    .id
                    .as_ref()
                    .map(|id| id == request_id)
                    .unwrap_or(false)
                {
                    found_id = true;
                    // 如果 URL 还没有提取，尝试从当前行提取
                    if entry.url.is_none() {
                        if let Some(ref mut entry) = current_entry {
                            entry.url = extract_url_from_line(&line);
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

/// 提取日志条目的响应内容（从 "response:" 开始到空行结束）
///
/// 匹配 shell 脚本 qkfind.sh 的逻辑：
/// 1. 查找包含 `#<request_id>` 的行（prev）
/// 2. 查找下一行包含 `response:` 的行
/// 3. 提取 `response: ` 之后的内容，直到空行
pub fn extract_response_content(log_file: &Path, request_id: &str) -> Result<String> {
    let reader = open_log_file(log_file)?;
    let mut lines = reader.lines();
    let mut response_lines = Vec::new();
    let mut prev_line = String::new();
    let mut in_response = false;

    while let Some(Ok(line)) = lines.next() {
        // 检查是否包含 response:
        if line.contains("response:") {
            // 检查上一行是否包含请求 ID
            if prev_line.contains(&format!("#{}", request_id)) {
                in_response = true;
                // 提取 "response: " 之后的内容
                if let Some(response_start) = line.find("response:") {
                    let response_content = &line[response_start + 9..].trim_start();
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

/// 查找日志文件
/// 在指定目录中查找 flutter-api*.log 文件
pub fn find_log_file(base_dir: &Path) -> Result<PathBuf> {
    // 尝试查找 flutter-api*.log 文件
    let log_files: Vec<_> = std::fs::read_dir(base_dir)
        .context("Failed to read directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        return name.starts_with("flutter-api") && name.ends_with(".log");
                    }
                }
            }
            false
        })
        .map(|entry| entry.path())
        .collect();

    if let Some(log_file) = log_files.first() {
        Ok(log_file.clone())
    } else {
        // 如果没找到，返回默认路径
        Ok(base_dir.join("flutter-api.log"))
    }
}

/// 获取日志文件路径
/// 根据 JIRA ID 自动解析日志文件路径
pub fn get_log_file_path(jira_id: &str) -> Result<PathBuf> {
    let settings = Settings::load();

    // 获取配置的基础目录
    let base_dir_str = settings.log_download_base_dir;

    // 展开 ~ 路径
    let base_dir_path = expand_path(&base_dir_str)?;

    // 每个 JIRA ticket 使用独立的子目录
    let ticket_dir = base_dir_path.join(jira_id);

    // 从 Settings 获取日志输出文件夹名称
    let extract_dir = if !settings.log_output_folder_name.is_empty() {
        ticket_dir.join(&settings.log_output_folder_name)
    } else {
        ticket_dir.join("merged")
    };

    // 如果新位置存在，使用新位置
    if extract_dir.exists() {
        return find_log_file(&extract_dir);
    }

    // 向后兼容：尝试查找旧位置
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let home_path = PathBuf::from(&home);
    let old_base_dir = if !settings.log_output_folder_name.is_empty() {
        home_path.join(format!(
            "Downloads/logs_{}/{}/merged",
            jira_id, settings.log_output_folder_name
        ))
    } else {
        home_path.join(format!("Downloads/logs_{}/merged", jira_id))
    };

    if old_base_dir.exists() {
        return find_log_file(&old_base_dir);
    }

    // 如果旧位置也不存在，尝试在旧目录下查找
    let old_logs_dir = home_path.join(format!("Downloads/logs_{}", jira_id));
    if old_logs_dir.exists() {
        // 查找 merged 或任何包含 flutter-api*.log 的目录
        if let Ok(entries) = std::fs::read_dir(&old_logs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let potential_log_file = path.join("flutter-api.log");
                    if potential_log_file.exists() {
                        return Ok(potential_log_file);
                    }
                }
            }
        }
    }

    // 如果都找不到，返回新位置的默认路径
    find_log_file(&extract_dir)
}
