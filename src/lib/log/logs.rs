use anyhow::{Context, Result};
use regex::Regex;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::{Jira, Logger, Settings};
use crate::jira::helpers::get_auth;

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

    /// 从 Jira ticket 下载日志附件
    /// 返回下载的基础目录路径
    pub fn download_from_jira(
        jira_id: &str,
        log_output_folder_name: Option<&str>,
        download_all_attachments: bool,
    ) -> Result<PathBuf> {
        // 1. 确定输出目录
        let settings = Settings::load();
        let base_dir_str = settings.log_download_base_dir;

        // 展开 ~ 路径
        let base_dir = if let Some(rest) = base_dir_str.strip_prefix("~/") {
            let home = env::var("HOME").context("HOME environment variable not set")?;
            PathBuf::from(home).join(rest)
        } else if base_dir_str == "~" {
            let home = env::var("HOME").context("HOME environment variable not set")?;
            PathBuf::from(home)
        } else {
            PathBuf::from(&base_dir_str)
        };

        // 每个 JIRA ticket 使用独立的子目录
        let base_dir = base_dir.join(jira_id);

        // 如果目录已存在，删除它
        if base_dir.exists() {
            std::fs::remove_dir_all(&base_dir).context("Failed to remove existing directory")?;
        }

        std::fs::create_dir_all(&base_dir).context("Failed to create output directory")?;

        let download_dir = base_dir.join("downloads");
        std::fs::create_dir_all(&download_dir).context("Failed to create download directory")?;

        // 2. 获取附件列表
        let attachments =
            Jira::get_attachments(jira_id).context("Failed to get attachments from Jira")?;

        if attachments.is_empty() {
            anyhow::bail!("No attachments found for {}", jira_id);
        }

        // 3. 过滤日志附件（log.zip, log.z01, etc.）
        let log_attachments: Vec<_> = attachments
            .iter()
            .filter(|a| {
                a.filename.starts_with("log.")
                    && (a.filename == "log.zip" || a.filename.starts_with("log.z"))
            })
            .collect();

        // 4. 下载附件
        if download_all_attachments {
            // 下载所有附件
            for attachment in &attachments {
                let file_path = download_dir.join(&attachment.filename);
                Self::download_file(&attachment.content_url, &file_path)?;
            }
        } else {
            // 只下载日志附件
            if log_attachments.is_empty() {
                anyhow::bail!("No log attachments found for {}", jira_id);
            }

            for attachment in &log_attachments {
                let file_path = download_dir.join(&attachment.filename);
                Self::download_file(&attachment.content_url, &file_path)?;
            }
        }

        // 5. 处理日志附件（合并分片、解压）
        let log_zip = download_dir.join("log.zip");
        if log_zip.exists() {
            // 检查是否有分片文件
            let has_split_files = std::fs::read_dir(&download_dir)?
                .filter_map(|e| e.ok())
                .any(|e| {
                    if let Some(name) = e.file_name().to_str() {
                        name.starts_with("log.z")
                            && name.len() == 8
                            && name[6..].parse::<u8>().is_ok()
                    } else {
                        false
                    }
                });

            if has_split_files {
                Self::merge_split_zips(&download_dir)?;
            } else {
                // 单个 zip 文件，直接复制为 merged.zip
                let merged_zip = download_dir.join("merged.zip");
                std::fs::copy(&log_zip, &merged_zip)
                    .context("Failed to copy log.zip to merged.zip")?;
            }

            // 解压文件
            let extract_dir = if let Some(folder_name) = log_output_folder_name {
                base_dir.join(folder_name)
            } else {
                base_dir.join("merged")
            };

            let merged_zip = download_dir.join("merged.zip");
            if merged_zip.exists() {
                Self::extract_zip(&merged_zip, &extract_dir)?;
            }
        } else if !download_all_attachments {
            // 如果没有日志附件且不是下载所有附件，返回错误
            anyhow::bail!("log.zip not found after download");
        }

        Ok(base_dir)
    }

    /// 下载单个文件
    fn download_file(url: &str, output_path: &Path) -> Result<()> {
        // 获取 Jira 认证信息
        let (email, api_token) = get_auth()?;

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        // 添加 Basic Auth 认证
        let mut response = client
            .get(url)
            .basic_auth(&email, Some(&api_token))
            .send()
            .with_context(|| format!("Failed to download: {}", url))?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        let mut file = File::create(output_path)
            .with_context(|| format!("Failed to create file: {:?}", output_path))?;

        std::io::copy(&mut response, &mut file)
            .with_context(|| format!("Failed to write file: {:?}", output_path))?;

        Ok(())
    }

    /// 查找请求 ID 并发送到 Streamock 服务
    /// 返回提取的响应内容
    pub fn find_and_send_to_streamock(
        log_file: &Path,
        request_id: &str,
        jira_id: Option<&str>,
        jira_service_address: Option<&str>,
        streamock_url: Option<&str>,
    ) -> Result<String> {
        // 1. 提取响应内容
        let response_content = Self::extract_response_content(log_file, request_id)
            .context("Failed to extract response content")?;

        // 2. 获取日志条目的 URL 信息（用于生成 name）
        let entry = Self::find_request_id(log_file, request_id)?;
        let name = if let Some(entry) = entry {
            if let Some(url) = entry.url {
                // 提取 URL 的最后两段路径
                let url_parts: Vec<&str> = url.split('/').collect();
                if url_parts.len() >= 2 {
                    format!(
                        "#{} {}",
                        request_id,
                        url_parts[url_parts.len() - 2..].join("/")
                    )
                } else {
                    format!("#{} {}", request_id, url_parts.last().unwrap_or(&"unknown"))
                }
            } else {
                format!("#{} unknown", request_id)
            }
        } else {
            format!("#{} unknown", request_id)
        };

        // 3. 生成 domain
        let domain = if let Some(jira_id) = jira_id {
            if let Some(jira_service) = jira_service_address {
                format!("{}/browse/{}", jira_service, jira_id)
            } else {
                format!("jira/{}", jira_id)
            }
        } else {
            log_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        };

        // 4. 生成时间戳
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let formatted_timestamp = format!("Unix timestamp: {}", now);

        // 5. 创建 JSON payload
        let payload = serde_json::json!({
            "encodedKey": "",
            "data": response_content,
            "combineLine": "",
            "separator": "",
            "domain": domain,
            "name": name,
            "timestamp": formatted_timestamp
        });

        // 6. 发送到 Streamock 服务
        let streamock_url = streamock_url.unwrap_or("http://localhost:3001/api/submit");
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(streamock_url)
            .header("Content-Type", "application/json")
            .header("Connection", "keep-alive")
            .json(&payload)
            .send()
            .context("Failed to send request to Streamock")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Streamock request failed: {} - {}", status, body);
        }

        Ok(response_content)
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
        let base_dir_path = if let Some(rest) = base_dir_str.strip_prefix("~/") {
            let home = env::var("HOME").context("HOME environment variable not set")?;
            PathBuf::from(home).join(rest)
        } else if base_dir_str == "~" {
            let home = env::var("HOME").context("HOME environment variable not set")?;
            PathBuf::from(home)
        } else {
            PathBuf::from(&base_dir_str)
        };

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
            return Self::find_log_file(&extract_dir);
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
            return Self::find_log_file(&old_base_dir);
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
        Self::find_log_file(&extract_dir)
    }

    /// 获取基础目录路径
    /// 展开 ~ 路径并返回完整的基础目录路径
    fn get_base_dir_path() -> Result<PathBuf> {
        let settings = Settings::load();
        let base_dir_str = settings.log_download_base_dir;

        // 展开 ~ 路径
        if let Some(rest) = base_dir_str.strip_prefix("~/") {
            let home = env::var("HOME").context("HOME environment variable not set")?;
            Ok(PathBuf::from(home).join(rest))
        } else if base_dir_str == "~" {
            let home = env::var("HOME").context("HOME environment variable not set")?;
            Ok(PathBuf::from(home))
        } else {
            Ok(PathBuf::from(&base_dir_str))
        }
    }

    /// 计算目录大小和文件数量
    fn calculate_dir_info(dir: &Path) -> Result<(u64, usize)> {
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

    /// 格式化文件大小
    fn format_size(bytes: u64) -> String {
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

    /// 列出目录内容
    fn list_dir_contents(dir: &Path) -> Result<Vec<PathBuf>> {
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

    /// 清除整个基础目录
    ///
    /// # 参数
    ///
    /// * `dry_run` - 如果为 true，只预览操作，不实际删除
    /// * `list_only` - 如果为 true，只列出将要删除的内容
    ///
    /// # 返回
    ///
    /// 返回是否实际执行了删除操作
    pub fn clean_base_dir(dry_run: bool, list_only: bool) -> Result<bool> {
        let base_dir = Self::get_base_dir_path()?;

        if !base_dir.exists() {
            crate::log_info!("Base directory does not exist: {:?}", base_dir);
            return Ok(false);
        }

        let (size, file_count) = Self::calculate_dir_info(&base_dir)?;

        if list_only {
            crate::log_info!("Base directory: {:?}", base_dir);
            crate::log_info!("Total size: {}", Self::format_size(size));
            crate::log_info!("Total files: {}", file_count);
            crate::log_info!("\nContents:");
            let contents = Self::list_dir_contents(&base_dir)?;
            for path in contents {
                if path.is_file() {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        crate::log_info!("  📄 {} ({})", path.display(), Self::format_size(metadata.len()));
                    } else {
                        crate::log_info!("  📄 {}", path.display());
                    }
                } else if path.is_dir() {
                    crate::log_info!("  📁 {}", path.display());
                }
            }
            return Ok(false);
        }

        if dry_run {
            crate::log_info!("[DRY RUN] Would delete base directory: {:?}", base_dir);
            crate::log_info!("[DRY RUN] Total size: {}", Self::format_size(size));
            crate::log_info!("[DRY RUN] Total files: {}", file_count);
            return Ok(false);
        }

        // 显示将要删除的信息
        crate::log_info!("Base directory: {:?}", base_dir);
        crate::log_info!("Total size: {}", Self::format_size(size));
        crate::log_info!("Total files: {}", file_count);

        // 需要确认
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete the entire base directory? This will remove {} files ({}).",
                file_count,
                Self::format_size(size)
            ))
            .default(false)
            .interact()
            .context("Failed to get confirmation")?;

        if !confirmed {
            crate::log_info!("Clean operation cancelled.");
            return Ok(false);
        }

        // 执行删除
        std::fs::remove_dir_all(&base_dir)
            .context(format!("Failed to delete base directory: {:?}", base_dir))?;

        crate::log_success!("Base directory deleted successfully: {:?}", base_dir);
        Ok(true)
    }

    /// 清除特定 JIRA ID 的目录
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA ticket ID
    /// * `dry_run` - 如果为 true，只预览操作，不实际删除
    /// * `list_only` - 如果为 true，只列出将要删除的内容
    ///
    /// # 返回
    ///
    /// 返回是否实际执行了删除操作
    pub fn clean_jira_dir(jira_id: &str, dry_run: bool, list_only: bool) -> Result<bool> {
        let base_dir = Self::get_base_dir_path()?;
        let jira_dir = base_dir.join(jira_id);

        if !jira_dir.exists() {
            crate::log_info!("Directory does not exist for {}: {:?}", jira_id, jira_dir);
            return Ok(false);
        }

        let (size, file_count) = Self::calculate_dir_info(&jira_dir)?;

        if list_only {
            crate::log_info!("JIRA ID: {}", jira_id);
            crate::log_info!("Directory: {:?}", jira_dir);
            crate::log_info!("Total size: {}", Self::format_size(size));
            crate::log_info!("Total files: {}", file_count);
            crate::log_info!("\nContents:");
            let contents = Self::list_dir_contents(&jira_dir)?;
            for path in contents {
                if path.is_file() {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        crate::log_info!("  📄 {} ({})", path.display(), Self::format_size(metadata.len()));
                    } else {
                        crate::log_info!("  📄 {}", path.display());
                    }
                } else if path.is_dir() {
                    crate::log_info!("  📁 {}", path.display());
                }
            }
            return Ok(false);
        }

        if dry_run {
            crate::log_info!("[DRY RUN] Would delete directory for {}: {:?}", jira_id, jira_dir);
            crate::log_info!("[DRY RUN] Total size: {}", Self::format_size(size));
            crate::log_info!("[DRY RUN] Total files: {}", file_count);
            return Ok(false);
        }

        // 显示将要删除的信息
        crate::log_info!("JIRA ID: {}", jira_id);
        crate::log_info!("Directory: {:?}", jira_dir);
        crate::log_info!("Total size: {}", Self::format_size(size));
        crate::log_info!("Total files: {}", file_count);

        // 需要确认
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete the directory for {}? This will remove {} files ({}).",
                jira_id,
                file_count,
                Self::format_size(size)
            ))
            .default(false)
            .interact()
            .context("Failed to get confirmation")?;

        if !confirmed {
            crate::log_info!("Clean operation cancelled.");
            return Ok(false);
        }

        // 执行删除
        std::fs::remove_dir_all(&jira_dir)
            .context(format!("Failed to delete directory for {}: {:?}", jira_id, jira_dir))?;

        crate::log_success!("Directory deleted successfully for {}: {:?}", jira_id, jira_dir);
        Ok(true)
    }
}
