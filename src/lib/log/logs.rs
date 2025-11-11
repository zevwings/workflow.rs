use anyhow::{Context, Result};
use dialoguer::Confirm;
use regex::Regex;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::jira::helpers::get_auth;
use crate::{log_debug, log_info, log_success, Jira, Logger, Settings};

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
    ///
    /// 匹配 shell 脚本 qkfind.sh 的逻辑：
    /// 1. 查找包含 `#<request_id>` 的行
    /// 2. 从该行中提取 URL（如果存在）
    /// 3. 返回包含 ID 和 URL 的条目
    pub fn find_request_id(log_file: &Path, request_id: &str) -> Result<Option<LogEntry>> {
        let file = File::open(log_file)
            .with_context(|| format!("Failed to open log file: {:?}", log_file))?;

        let reader = BufReader::new(file);
        let mut current_entry: Option<LogEntry> = None;
        let mut found_id = false;

        for line_result in reader.lines() {
            let line = line_result.context("Failed to read line")?;

            // 检查是否包含请求 ID（匹配 shell 脚本：$0 ~ "#" rid）
            if line.contains(&format!("#{}", request_id)) {
                // 解析条目（提取 ID 和 URL）
                current_entry = Self::parse_log_entry(&line)?;

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
                                entry.url = Self::extract_url_from_line(&line);
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
        let file = File::open(log_file)
            .with_context(|| format!("Failed to open log file: {:?}", log_file))?;

        let reader = BufReader::new(file);
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

        if response_lines.is_empty() {
            anyhow::bail!("No response content found for request ID: {}", request_id);
        }

        Ok(response_lines.join("\n"))
    }

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
        let file = File::open(log_file)
            .with_context(|| format!("Failed to open log file: {:?}", log_file))?;

        let reader = BufReader::new(file);
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();
        let mut printed_ids = std::collections::HashSet::new();
        let mut current_entry: Option<LogEntry> = None;
        let mut found_in_current_block = false;

        // 用于检测 api.log 格式的条目（包含 `#<数字> <HTTP方法>` 的模式）
        let api_log_entry_pattern = Regex::new(r"#\d+\s+(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)").ok();

        for line_result in reader.lines() {
            let line = line_result.context("Failed to read line")?;
            let line_lower = line.to_lowercase();

            // 检查是否是新条目的开始
            let is_new_entry = if line.starts_with("💡") {
                // flutter-api.log 格式：以 💡 开头
                true
            } else if let Some(pattern) = &api_log_entry_pattern {
                // api.log 格式：包含 `#<数字> <HTTP方法>` 的模式
                pattern.is_match(&line)
            } else {
                false
            };

            if is_new_entry {
                // 如果之前的条目匹配，保存它（避免重复）
                if found_in_current_block {
                    if let Some(entry) = current_entry.take() {
                        if let Some(ref id) = entry.id {
                            if !printed_ids.contains(id) {
                                results.push(entry.clone());
                                printed_ids.insert(id.clone());
                            }
                        }
                    }
                }

                // 解析新条目
                current_entry = Self::parse_log_entry(&line)?;
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
                        entry.url = Self::extract_url_from_line(&line);
                    }
                }
            }

            // 空行表示块结束
            if line.trim().is_empty() {
                // 如果当前块匹配，保存结果
                if found_in_current_block {
                    if let Some(entry) = current_entry.take() {
                        if let Some(ref id) = entry.id {
                            if !printed_ids.contains(id) {
                                results.push(entry.clone());
                                printed_ids.insert(id.clone());
                            }
                        }
                    }
                }
                // 重置状态
                current_entry = None;
                found_in_current_block = false;
            }
        }

        // 检查最后一个条目
        if found_in_current_block {
            if let Some(entry) = current_entry {
                if let Some(ref id) = entry.id {
                    if !printed_ids.contains(id) {
                        results.push(entry);
                    }
                }
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
    ///
    /// 匹配 shell 脚本的逻辑：
    /// 1. 首先尝试匹配 HTTP 方法（GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS）后的 URL
    /// 2. 如果没有找到，尝试匹配格式：`数字 https://...`
    /// 3. 清理 URL（移除引号、单引号、空格、逗号、右花括号等）
    fn extract_url_from_line(line: &str) -> Option<String> {
        // 清理 URL 的辅助函数
        fn clean_url(url: &str) -> String {
            url.trim_end_matches(['"', '\'', ' ', ',', '}']).to_string()
        }

        // 方法 1: 查找 HTTP 方法后的 URL
        // 匹配: GET https://... 或 POST https://... 等
        // 使用字符类匹配非空白、非引号、非逗号的字符（单引号和右花括号通过 clean_url 处理）
        let method_pattern =
            Regex::new("(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\\s+(https?://[^\\s\",]+)").ok()?;
        if let Some(caps) = method_pattern.captures(line) {
            if let Some(url_match) = caps.get(2) {
                return Some(clean_url(url_match.as_str()));
            }
        }

        // 方法 2: 查找格式 `数字 https://...`
        let number_url_pattern = Regex::new("\\d+\\s+(https?://[^\\s\",]+)").ok()?;
        if let Some(caps) = number_url_pattern.captures(line) {
            if let Some(url_match) = caps.get(1) {
                return Some(clean_url(url_match.as_str()));
            }
        }

        // 方法 3: 直接匹配 HTTP URL（向后兼容）
        let url_re = Regex::new(r#"https?://[^\s",]+"#).ok()?;
        url_re.find(line).map(|m| clean_url(m.as_str()))
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

        // 创建 downloads 子目录（与 shell 脚本一致）
        let download_dir = base_dir.join("downloads");

        // 如果目录已存在，删除它
        if base_dir.exists() {
            std::fs::remove_dir_all(&base_dir).context("Failed to remove existing directory")?;
        }

        std::fs::create_dir_all(&download_dir).context("Failed to create output directory")?;

        // 2. 获取附件列表
        let attachments: Vec<crate::JiraAttachment> =
            Jira::get_attachments(jira_id).context("Failed to get attachments from Jira")?;

        if attachments.is_empty() {
            anyhow::bail!("No attachments found for {}", jira_id);
        }

        // 调试：显示所有附件
        log_debug!("Found {} attachment(s):", attachments.len());
        for attachment in &attachments {
            log_debug!("  - {}", attachment.filename);
        }

        // 3. 过滤日志附件
        // 匹配规则（与 shell 脚本的 awk 模式一致）：
        // 1. log.zip 或 log.z[0-9]+ 格式的文件（如 log.zip, log.z01, log.z02 等）
        // 2. 以 .log 结尾的文件（如 any_file.log, error.log 等）
        // 3. 以 .txt 结尾的文件（如 metric0.txt, log0.txt, network3.txt 等）
        // Shell 脚本使用: /^[[:space:]]*[0-9]+\. log\.(zip|z[0-9]+)[[:space:]]*$/
        // 我们简化匹配：log\.(zip|z[0-9]+) 或 log\d*\.(zip|z[0-9]+)
        let log_zip_pattern = Regex::new(r"^log\.(zip|z\d+)$")?;
        let log_attachments: Vec<_> = attachments
            .iter()
            .filter(|a| {
                // 匹配 log.zip 或 log.z[0-9]+ 格式（与 shell 脚本一致）
                // 例如：log.zip, log.z01, log.z02 等
                let matches_log_zip = log_zip_pattern.is_match(&a.filename);
                // 匹配所有以 .log 结尾的文件
                // 例如：any_file.log, error.log, debug.log 等
                let matches_log_ext = a.filename.ends_with(".log");
                // 匹配所有以 .txt 结尾的文件
                // 例如：metric0.txt, log0.txt, network3.txt, any_file.txt 等
                let matches_txt_ext = a.filename.ends_with(".txt");

                matches_log_zip || matches_log_ext || matches_txt_ext
            })
            .collect();

        // 调试：显示过滤后的日志附件
        if !log_attachments.is_empty() {
            log_debug!("Filtered {} log attachment(s):", log_attachments.len());
            for attachment in &log_attachments {
                log_debug!("  - {}", attachment.filename);
            }
        }

        // 4. 下载附件
        if download_all_attachments {
            // 下载所有附件到 downloads 目录
            for attachment in &attachments {
                let file_path = download_dir.join(&attachment.filename);
                Self::download_file(&attachment.content_url, &file_path)?;
            }
        } else {
            // 只下载日志附件
            if log_attachments.is_empty() {
                anyhow::bail!("No log attachments found for {}", jira_id);
            }

            // 预先编译正则表达式，避免在循环中重复编译
            let link_pattern = Regex::new(r#"#\s*\[([^|]+)\|([^\]]+)\]"#).unwrap();

            // 预先获取描述中的原始 URL 映射，避免重复解析
            let mut original_urls: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            if let Ok(issue) = Jira::get_ticket_info(jira_id) {
                if let Some(description) = &issue.fields.description {
                    for cap in link_pattern.captures_iter(description) {
                        if let (Some(filename_match), Some(url_match)) = (cap.get(1), cap.get(2)) {
                            let filename = filename_match.as_str().trim().to_string();
                            let url = url_match.as_str().trim().to_string();
                            if url.contains("cloudfront.net") {
                                original_urls.insert(filename, url);
                            }
                        }
                    }
                }
            }

            // 尝试从 Jira API 的附件列表中查找匹配的文件名
            let mut api_attachments_map: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            if let Ok(issue) = Jira::get_ticket_info(jira_id) {
                if let Some(api_attachments) = &issue.fields.attachment {
                    for api_att in api_attachments {
                        api_attachments_map
                            .insert(api_att.filename.clone(), api_att.content_url.clone());
                    }
                }
            }

            let mut failed_attachments = Vec::new();
            for attachment in &log_attachments {
                let file_path = download_dir.join(&attachment.filename);

                // 首先尝试使用当前的 URL
                let (mut download_success, original_error) = match Self::download_file(
                    &attachment.content_url,
                    &file_path,
                ) {
                    Ok(()) => {
                        log_success!("Downloaded: {}", attachment.filename);
                        (true, None)
                    }
                    Err(e) => {
                        log_debug!(
                            "Warning: Failed to download {}: {}",
                            attachment.filename,
                            e
                        );
                        let error_msg = format!("{}", e);

                        // 如果当前 URL 是 CloudFront URL，尝试多种方式：
                        // 1. 从 Jira API 附件列表中查找匹配的文件名
                        // 2. 从 CloudFront URL 中提取附件 ID 并构建 Jira API URL
                        let success = if attachment.content_url.contains("cloudfront.net") {
                            let mut success = false;

                            // 方式 1: 从 API 附件列表中查找
                            if let Some(api_url) = api_attachments_map.get(&attachment.filename) {
                                log_debug!(
                                    "Trying Jira API URL for {}: {}",
                                    attachment.filename,
                                    api_url
                                );
                                match Self::download_file(api_url, &file_path) {
                                    Ok(()) => {
                                        log_success!(
                                            "Downloaded: {} (using Jira API URL)",
                                            attachment.filename
                                        );
                                        success = true;
                                    }
                                    Err(e2) => {
                                        log_debug!(
                                            "Also failed with Jira API URL: {}",
                                            e2
                                        );
                                    }
                                }
                            }

                            // 方式 2: 从 CloudFront URL 中提取附件 ID 并构建 Jira API URL
                            if !success {
                                if let Some(attachment_id) =
                                    crate::jira::ticket::extract_attachment_id_from_url(
                                        &attachment.content_url,
                                    )
                                {
                                    if let Ok(base_url) = crate::jira::helpers::get_base_url() {
                                        let jira_api_url = format!(
                                            "{}/attachment/content/{}",
                                            base_url, attachment_id
                                        );
                                        log_debug!("Trying Jira API URL from attachment ID {} for {}: {}",
                                            attachment_id, attachment.filename, jira_api_url);
                                        match Self::download_file(&jira_api_url, &file_path) {
                                            Ok(()) => {
                                                log_success!("Downloaded: {} (using Jira API URL from attachment ID)", attachment.filename);
                                                success = true;
                                            }
                                            Err(e2) => {
                                                log_debug!("Also failed with Jira API URL from attachment ID: {}", e2);
                                            }
                                        }
                                    }
                                }
                            }

                            success
                        } else {
                            false
                        };

                        (success, Some(error_msg))
                    }
                };

                // 如果还是失败，尝试使用原始 CloudFront URL（如果不同）
                if !download_success {
                    if let Some(original_url) = original_urls.get(&attachment.filename) {
                        if original_url != &attachment.content_url {
                            log_debug!(
                                "Retrying with original CloudFront URL for {}",
                                attachment.filename
                            );
                            download_success = match Self::download_file(original_url, &file_path) {
                                Ok(()) => {
                                    log_success!(
                                        "Downloaded: {} (using original CloudFront URL)",
                                        attachment.filename
                                    );
                                    true
                                }
                                Err(e2) => {
                                    log_debug!(
                                        "Warning: Also failed with original CloudFront URL: {}",
                                        e2
                                    );
                                    false
                                }
                            };
                        }
                    }
                }

                if !download_success {
                    // 使用保存的原始错误信息
                    if let Some(error) = original_error {
                        failed_attachments
                            .push((attachment.filename.clone(), anyhow::anyhow!("{}", error)));
                    }
                }
            }

            // 如果有失败的附件，显示警告但不中断整个流程
            if !failed_attachments.is_empty() {
                log_info!(
                    "\n⚠️  Warning: {} attachment(s) failed to download:",
                    failed_attachments.len()
                );
                for (filename, error) in &failed_attachments {
                    log_info!("  - {}: {}", filename, error);
                }
            }
        }

        // 5. 处理日志附件（合并分片、解压）
        // 检查是否有 log.zip 或分片文件（与 shell 脚本一致）
        let log_zip = download_dir.join("log.zip");
        let log_z01 = download_dir.join("log.z01");

        if log_zip.exists() {
            // 检查是否有分片文件（与 shell 脚本一致：检查 log.z01）
            if log_z01.exists() {
                // 检测到分片文件，需要合并（与 shell 脚本一致）
                log_debug!("检测到分片文件，需要合并...");
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
            // 检查是否有成功下载的日志文件（.txt, .log 等）
            let has_log_files = std::fs::read_dir(&download_dir)?
                .filter_map(|e| e.ok())
                .any(|e| {
                    if let Some(name) = e.file_name().to_str() {
                        name.ends_with(".txt") || name.ends_with(".log") || name.ends_with(".zip")
                    } else {
                        false
                    }
                });

            if !has_log_files {
                // 如果没有日志附件且不是下载所有附件，返回错误
                anyhow::bail!(
                    "No log files found after download. All log attachments failed to download."
                );
            }
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

        // 判断是否是 CloudFront 签名 URL（包含 Expires 和 Signature 参数）
        // CloudFront 签名 URL 通常不需要 Basic Auth，或者 Basic Auth 会干扰签名验证
        let is_cloudfront_signed_url = url.contains("cloudfront.net")
            && url.contains("Expires=")
            && url.contains("Signature=");

        // 获取 Jira base URL 用于 Referer 头
        let jira_base_url = crate::jira::helpers::get_base_url().ok();

        let mut response = if is_cloudfront_signed_url {
            // CloudFront 签名 URL，先尝试不使用 Basic Auth，但添加 Referer 头
            log_debug!("Using CloudFront signed URL without Basic Auth");
            let mut request = client.get(url);

            // 添加 Referer 头，可能有助于 CloudFront 验证
            if let Some(ref base_url) = jira_base_url {
                request = request.header("Referer", base_url);
            }

            request
                .send()
                .with_context(|| format!("Failed to download: {}", url))?
        } else {
            // Jira API URL，使用 Basic Auth
            client
                .get(url)
                .basic_auth(&email, Some(&api_token))
                .send()
                .with_context(|| format!("Failed to download: {}", url))?
        };

        if !response.status().is_success() {
            // 如果 CloudFront URL 失败，尝试使用 Basic Auth
            if is_cloudfront_signed_url {
                let status = response.status();
                log_debug!(
                    "CloudFront URL failed (status: {}), retrying with Basic Auth",
                    status
                );

                // 尝试读取响应体以获取更多错误信息
                let error_text = response.text().unwrap_or_default();
                if !error_text.is_empty() {
                    let preview = if error_text.len() > 200 {
                        format!("{}...", &error_text[..200])
                    } else {
                        error_text.clone()
                    };
                    log_debug!("Error response: {}", preview);
                }

                let mut request = client.get(url);
                // 添加 Referer 头
                if let Some(ref base_url) = jira_base_url {
                    request = request.header("Referer", base_url);
                }

                response = request
                    .basic_auth(&email, Some(&api_token))
                    .send()
                    .with_context(|| format!("Failed to download with Basic Auth: {}", url))?;

                if !response.status().is_success() {
                    let status = response.status();
                    // 尝试读取响应体以获取更多错误信息
                    let error_text = response.text().unwrap_or_default();
                    let error_msg = if !error_text.is_empty() {
                        let preview = if error_text.len() > 200 {
                            format!("{}...", &error_text[..200])
                        } else {
                            error_text
                        };
                        format!("Download failed with status: {} - {}", status, preview)
                    } else {
                        format!("Download failed with status: {}", status)
                    };

                    anyhow::bail!("{}", error_msg);
                }
            } else {
                let status = response.status();
                // 尝试读取响应体以获取更多错误信息
                let error_text = response.text().unwrap_or_default();
                let error_msg = if !error_text.is_empty() {
                    let preview = if error_text.len() > 200 {
                        format!("{}...", &error_text[..200])
                    } else {
                        error_text
                    };
                    format!("Download failed with status: {} - {}", status, preview)
                } else {
                    format!("Download failed with status: {}", status)
                };

                anyhow::bail!("{}", error_msg);
            }
        }

        let mut file = File::create(output_path)
            .with_context(|| format!("Failed to create file: {:?}", output_path))?;

        std::io::copy(&mut response, &mut file)
            .with_context(|| format!("Failed to write file: {:?}", output_path))?;

        Ok(())
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
            log_info!("Base directory does not exist: {:?}", base_dir);
            return Ok(false);
        }

        let (size, file_count) = Self::calculate_dir_info(&base_dir)?;

        if list_only {
            log_info!("Base directory: {:?}", base_dir);
            log_info!("Total size: {}", Self::format_size(size));
            log_info!("Total files: {}", file_count);
            log_info!("\nContents:");
            let contents = Self::list_dir_contents(&base_dir)?;
            for path in contents {
                if path.is_file() {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        log_info!(
                            "  📄 {} ({})",
                            path.display(),
                            Self::format_size(metadata.len())
                        );
                    } else {
                        log_info!("  📄 {}", path.display());
                    }
                } else if path.is_dir() {
                    log_info!("  📁 {}", path.display());
                }
            }
            return Ok(false);
        }

        if dry_run {
            log_info!("[DRY RUN] Would delete base directory: {:?}", base_dir);
            log_info!("[DRY RUN] Total size: {}", Self::format_size(size));
            log_info!("[DRY RUN] Total files: {}", file_count);
            return Ok(false);
        }

        // 显示将要删除的信息
        log_info!("Base directory: {:?}", base_dir);
        log_info!("Total size: {}", Self::format_size(size));
        log_info!("Total files: {}", file_count);

        // 需要确认
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
            log_info!("Clean operation cancelled.");
            return Ok(false);
        }

        // 执行删除
        std::fs::remove_dir_all(&base_dir)
            .context(format!("Failed to delete base directory: {:?}", base_dir))?;

        log_success!("Base directory deleted successfully: {:?}", base_dir);
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
            log_info!("Directory does not exist for {}: {:?}", jira_id, jira_dir);
            return Ok(false);
        }

        let (size, file_count) = Self::calculate_dir_info(&jira_dir)?;

        if list_only {
            log_info!("JIRA ID: {}", jira_id);
            log_info!("Directory: {:?}", jira_dir);
            log_info!("Total size: {}", Self::format_size(size));
            log_info!("Total files: {}", file_count);
            log_info!("\nContents:");
            let contents = Self::list_dir_contents(&jira_dir)?;
            for path in contents {
                if path.is_file() {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        log_info!(
                            "  📄 {} ({})",
                            path.display(),
                            Self::format_size(metadata.len())
                        );
                    } else {
                        log_info!("  📄 {}", path.display());
                    }
                } else if path.is_dir() {
                    log_info!("  📁 {}", path.display());
                }
            }
            return Ok(false);
        }

        if dry_run {
            log_info!(
                "[DRY RUN] Would delete directory for {}: {:?}",
                jira_id,
                jira_dir
            );
            log_info!("[DRY RUN] Total size: {}", Self::format_size(size));
            log_info!("[DRY RUN] Total files: {}", file_count);
            return Ok(false);
        }

        // 显示将要删除的信息
        log_info!("JIRA ID: {}", jira_id);
        log_info!("Directory: {:?}", jira_dir);
        log_info!("Total size: {}", Self::format_size(size));
        log_info!("Total files: {}", file_count);

        // 需要确认
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
            log_info!("Clean operation cancelled.");
            return Ok(false);
        }

        // 执行删除
        std::fs::remove_dir_all(&jira_dir).context(format!(
            "Failed to delete directory for {}: {:?}",
            jira_id, jira_dir
        ))?;

        log_success!(
            "Directory deleted successfully for {}: {:?}",
            jira_id,
            jira_dir
        );
        Ok(true)
    }
}
