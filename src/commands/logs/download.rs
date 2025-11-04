use crate::{log_info, log_success, log_warning, Jira, Logs};
use crate::settings::Settings;
use anyhow::{Context, Result};
use reqwest;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// 日志下载命令
pub struct LogsDownloadCommand;

impl LogsDownloadCommand {
    /// 下载 Jira ticket 的日志附件
    pub fn download(jira_id: &str) -> Result<PathBuf> {
        // 1. 确定输出目录
        let home = env::var("HOME").context("HOME environment variable not set")?;
        let base_dir = PathBuf::from(home).join(format!("Downloads/logs_{}", jira_id));

        // 如果目录已存在，删除它
        if base_dir.exists() {
            log_warning!("Removing existing directory: {:?}", base_dir);
            fs::remove_dir_all(&base_dir).context("Failed to remove existing directory")?;
        }

        fs::create_dir_all(&base_dir).context("Failed to create output directory")?;

        let download_dir = base_dir.join("downloads");
        fs::create_dir_all(&download_dir).context("Failed to create download directory")?;

        log_success!("Getting attachments for {}...", jira_id);

        // 2. 获取附件列表
        let attachments =
            Jira::get_attachments(jira_id).context("Failed to get attachments from Jira")?;

        // 过滤日志附件（log.zip, log.z01, etc.）
        let log_attachments: Vec<_> = attachments
            .iter()
            .filter(|a| {
                a.filename.starts_with("log.")
                    && (a.filename == "log.zip" || a.filename.starts_with("log.z"))
            })
            .collect();

        if log_attachments.is_empty() {
            anyhow::bail!("No log attachments found for {}", jira_id);
        }

        log_success!("Found {} log attachments:", log_attachments.len());
        for attachment in &log_attachments {
            log_info!("  - {}", attachment.filename);
        }

        // 3. 下载附件
        log_success!("\nDownloading attachments...");
        for attachment in &log_attachments {
            let file_path = download_dir.join(&attachment.filename);
            Self::download_file(&attachment.content_url, &file_path)?;
        }

        // 4. 处理单个 zip 文件或合并分片文件
        let log_zip = download_dir.join("log.zip");
        if !log_zip.exists() {
            anyhow::bail!("log.zip not found after download");
        }

        // 检查是否有分片文件
        let has_split_files = fs::read_dir(&download_dir)?
            .filter_map(|e| e.ok())
            .any(|e| {
                if let Some(name) = e.file_name().to_str() {
                    name.starts_with("log.z") && name.len() == 8 && name[6..].parse::<u8>().is_ok()
                } else {
                    false
                }
            });

        if has_split_files {
            log_success!("\nMerging split files...");
            let merged_zip = Logs::merge_split_zips(&download_dir)?;
            log_success!("Merged to: {:?}", merged_zip);
        } else {
            // 单个 zip 文件，直接复制为 merged.zip
            let merged_zip = download_dir.join("merged.zip");
            fs::copy(&log_zip, &merged_zip).context("Failed to copy log.zip to merged.zip")?;
            log_success!("Single zip file found, copied to merged.zip");
        }

        // 5. 解压文件（如果需要）
        let settings = Settings::get();
        let extract_dir = if !settings.log_output_folder_name.is_empty() {
            base_dir.join(&settings.log_output_folder_name)
        } else {
            base_dir.join("merged")
        };

        let merged_zip = download_dir.join("merged.zip");
        if merged_zip.exists() {
            log_success!("\nExtracting merged.zip...");
            Logs::extract_zip(&merged_zip, &extract_dir)?;
            log_success!("Extracted to: {:?}", extract_dir);
        }

        log_success!("\nDownload completed!");
        log_info!("Files located at: {:?}", base_dir);

        Ok(base_dir)
    }

    /// 下载单个文件
    fn download_file(url: &str, output_path: &Path) -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        let mut response = client
            .get(url)
            .send()
            .with_context(|| format!("Failed to download: {}", url))?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        let mut file = fs::File::create(output_path)
            .with_context(|| format!("Failed to create file: {:?}", output_path))?;

        std::io::copy(&mut response, &mut file)
            .with_context(|| format!("Failed to write file: {:?}", output_path))?;

        Ok(())
    }
}
