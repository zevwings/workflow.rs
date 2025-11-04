use anyhow::{Context, Result};
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::Logger;

/// 日志条目信息
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub id: Option<String>,
    pub url: Option<String>,
}

/// 日志处理模块
pub struct Logs;

impl Logs {
    /// 从日志文件中搜索请求 ID
    /// 返回包含该 ID 的条目信息
    pub fn find_request_id(log_file: &Path, request_id: &str) -> Result<Option<LogEntry>> {
        let file = File::open(log_file)
            .with_context(|| format!("Failed to open log file: {:?}", log_file))?;

        let reader = BufReader::new(file);
        let mut current_entry: Option<LogEntry> = None;
        let mut found_id = false;

        for line_result in reader.lines() {
            let line = line_result.context("Failed to read line")?;

            // 检查是否是新条目的开始（以 💡 开头）
            if line.starts_with("💡") {
                // 如果之前找到了匹配的条目，返回它
                if found_id {
                    break;
                }

                // 解析新条目
                current_entry = Self::parse_log_entry(&line)?;

                // 检查 ID 是否匹配
                if let Some(ref entry) = current_entry {
                    if entry
                        .id
                        .as_ref()
                        .map(|id| id == request_id)
                        .unwrap_or(false)
                    {
                        found_id = true;
                    }
                }
            } else if found_id {
                // 如果已找到匹配的条目，提取 URL（如果需要）
                if let Some(ref mut entry) = current_entry {
                    if entry.url.is_none() {
                        entry.url = Self::extract_url_from_line(&line);
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
    pub fn extract_response_content(log_file: &Path, request_id: &str) -> Result<String> {
        let file = File::open(log_file)
            .with_context(|| format!("Failed to open log file: {:?}", log_file))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut response_lines = Vec::new();
        let mut in_response = false;
        let mut found_request = false;

        while let Some(Ok(line)) = lines.next() {
            // 检查是否找到请求 ID
            if line.contains(&format!("#{}", request_id)) {
                found_request = true;
                continue;
            }

            // 如果找到了请求，开始查找 "response:"
            if found_request {
                if line.contains("response:") {
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

                // 如果在响应块中，收集内容直到空行
                if in_response {
                    if line.trim().is_empty() {
                        break; // 空行表示响应结束
                    }
                    response_lines.push(line);
                }
            }
        }

        if response_lines.is_empty() {
            anyhow::bail!("No response content found for request ID: {}", request_id);
        }

        Ok(response_lines.join("\n"))
    }

    /// 在日志文件中搜索关键词
    /// 返回匹配的请求信息列表（URL 和 ID）
    pub fn search_keyword(log_file: &Path, keyword: &str) -> Result<Vec<LogEntry>> {
        let file = File::open(log_file)
            .with_context(|| format!("Failed to open log file: {:?}", log_file))?;

        let reader = BufReader::new(file);
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();
        let mut current_entry: Option<LogEntry> = None;
        let mut found_in_current_block = false;

        for line_result in reader.lines() {
            let line = line_result.context("Failed to read line")?;
            let line_lower = line.to_lowercase();

            // 检查是否是新条目的开始
            if line.starts_with("💡") {
                // 如果之前的条目匹配，保存它
                if found_in_current_block {
                    if let Some(entry) = current_entry.take() {
                        results.push(entry);
                    }
                }

                // 解析新条目
                current_entry = Self::parse_log_entry(&line)?;
                found_in_current_block = false;
            } else if current_entry.is_some() {
                // 在当前块中搜索关键词
                if line_lower.contains(&keyword_lower) {
                    found_in_current_block = true;
                }

                // 提取 URL（如果需要）
                if let Some(ref mut entry) = current_entry {
                    if entry.url.is_none() {
                        entry.url = Self::extract_url_from_line(&line);
                    }
                }
            }

            // 空行表示块结束
            if line.trim().is_empty() && found_in_current_block {
                if let Some(entry) = current_entry.take() {
                    results.push(entry);
                }
                found_in_current_block = false;
            }
        }

        // 检查最后一个条目
        if found_in_current_block {
            if let Some(entry) = current_entry {
                results.push(entry);
            }
        }

        Ok(results)
    }

    /// 解析日志条目（从以 💡 开头的行）
    fn parse_log_entry(line: &str) -> Result<Option<LogEntry>> {
        // 提取 ID（#123 格式）
        let id_re = Regex::new(r"#(\d+)")?;
        let id = id_re
            .captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());

        // 尝试提取 URL
        let url = Self::extract_url_from_line(line);

        Ok(Some(LogEntry { id, url }))
    }

    /// 从行中提取 URL
    fn extract_url_from_line(line: &str) -> Option<String> {
        // 匹配 HTTP URL
        let url_re = Regex::new(r#"https?://[^\s"',]+"#).ok()?;
        url_re.find(line).map(|m| m.as_str().to_string())
    }

    /// 合并分片 zip 文件
    pub fn merge_split_zips(download_dir: &Path) -> Result<PathBuf> {
        let log_zip = download_dir.join("log.zip");
        if !log_zip.exists() {
            anyhow::bail!("log.zip not found in {:?}", download_dir);
        }

        // 查找所有分片文件（log.z01, log.z02, ...）
        let mut split_files: Vec<PathBuf> = WalkDir::new(download_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                if let Some(name) = e.file_name().to_str() {
                    name.starts_with("log.z") && name.len() == 8 && name[6..].parse::<u8>().is_ok()
                } else {
                    false
                }
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // 按文件名排序
        split_files.sort();

        if split_files.is_empty() {
            // 没有分片文件，直接复制 log.zip 为 merged.zip
            let merged_zip = download_dir.join("merged.zip");
            std::fs::copy(&log_zip, &merged_zip).context("Failed to copy log.zip to merged.zip")?;
            return Ok(merged_zip);
        }

        // 合并文件
        let merged_zip = download_dir.join("merged.zip");
        let mut output = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&merged_zip)
            .context("Failed to create merged.zip")?;

        // 写入 log.zip
        let mut input = File::open(&log_zip).context("Failed to open log.zip")?;
        std::io::copy(&mut input, &mut output).context("Failed to copy log.zip")?;

        // 写入所有分片文件
        for split_file in &split_files {
            let mut input = File::open(split_file)
                .with_context(|| format!("Failed to open {:?}", split_file))?;
            std::io::copy(&mut input, &mut output)
                .with_context(|| format!("Failed to copy {:?}", split_file))?;
        }

        output.flush().context("Failed to flush merged.zip")?;

        // 验证文件大小
        let expected_size: u64 = std::fs::metadata(&log_zip)?.len()
            + split_files
                .iter()
                .map(|f| std::fs::metadata(f).map(|m| m.len()).unwrap_or(0))
                .sum::<u64>();

        let actual_size = std::fs::metadata(&merged_zip)?.len();

        if actual_size != expected_size {
            Logger::print_warning(format!(
                "Merged file size mismatch (expected: {}, actual: {})",
                expected_size, actual_size
            ));
        }

        Ok(merged_zip)
    }

    /// 解压 zip 文件
    pub fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()> {
        let file = File::open(zip_path)
            .with_context(|| format!("Failed to open zip file: {:?}", zip_path))?;

        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("Failed to read zip archive: {:?}", zip_path))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .with_context(|| format!("Failed to read file {} from zip", i))?;

            let outpath = output_dir.join(file.name());

            if file.name().ends_with('/') {
                // 目录
                std::fs::create_dir_all(&outpath)
                    .with_context(|| format!("Failed to create directory: {:?}", outpath))?;
            } else {
                // 文件
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create parent directory: {:?}", parent)
                    })?;
                }

                let mut outfile = File::create(&outpath)
                    .with_context(|| format!("Failed to create file: {:?}", outpath))?;

                std::io::copy(&mut file, &mut outfile)
                    .with_context(|| format!("Failed to extract file: {:?}", outpath))?;
            }
        }

        Ok(())
    }
}
