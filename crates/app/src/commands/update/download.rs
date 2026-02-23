//! 下载和解压模块
//!
//! 提供文件下载和解压功能。

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    sync::Arc,
};

use client::{HttpClient, HttpClientHolder};
use di::Container;
use prompt::{info, spinner, success, warning, Spinner};
use toolkit::{
    archive, build_checksum_url, calculate_sha256, log_debug, log_info, parse_hash_from_content,
    verify_checksum, SizeExt,
};

use crate::commands::update::types::{GITHUB_DOWNLOAD_BASE, REPO_NAME, REPO_OWNER};

/// 构建下载 URL
///
/// 根据平台和版本号拼接下载链接。
pub fn build_download_url(version: impl AsRef<str>, platform: impl AsRef<str>) -> String {
    let version = version.as_ref();
    let platform = platform.as_ref();
    let extension = if platform.starts_with("Windows") {
        "zip"
    } else {
        "tar.gz"
    };

    format!(
        "{}/{}/{}/releases/download/v{}/workflow-{}-{}.{}",
        GITHUB_DOWNLOAD_BASE, REPO_OWNER, REPO_NAME, version, version, platform, extension
    )
}

/// 下载文件
///
/// 从指定 URL 下载文件到临时目录，显示下载进度。
pub fn download_file(
    url: impl AsRef<str>,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = url.as_ref();
    log_debug!("Download URL: {}", url);
    log_debug!("Saving to: {}", output_path.display());

    // 如果文件已存在且不完整，先删除它
    if output_path.exists() {
        if let Err(e) = fs::remove_file(output_path) {
            log_debug!("Failed to delete incomplete file: {}", e);
        }
    }

    // 获取 HTTP 客户端
    let http_client: Arc<dyn HttpClient> = Container::global()
        .get()
        .map_err(|e| format!("Failed to get HTTP client: {}", e))?;
    let client = HttpClientHolder::new(http_client);

    let size_bytes =
        spinner!("Downloading...").with(|| -> Result<usize, Box<dyn std::error::Error>> {
            let response = client
                .get(url)
                .send()
                .map_err(|e| format!("Failed to send HTTP request: {}", e))?;

            if !response.is_success() {
                return Err(format!("Download failed: HTTP {}", response.status).into());
            }

            let bytes = response.bytes();
            let len = bytes.len();
            let mut file = File::create(output_path)
                .map_err(|e| format!("Failed to create file {}: {}", output_path.display(), e))?;

            file.write_all(bytes.as_ref())
                .map_err(|e| format!("Failed to write to file: {}", e))?;

            Ok(len)
        })?;

    success!("Download completed");

    let size_human = (size_bytes as u64).to_size_string();
    log_info!(
        "Download completed | url={} output={} size={} ({})",
        url,
        output_path.display(),
        size_bytes,
        size_human
    );
    Ok(())
}

/// 验证文件校验和
///
/// 下载校验和文件并验证已下载文件的完整性。
pub fn verify_file_checksum(
    archive_path: &Path,
    download_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let checksum_url = build_checksum_url(download_url);

    log_debug!("Checksum URL: {}", checksum_url);

    // 获取 HTTP 客户端
    let http_client: Arc<dyn HttpClient> = Container::global()
        .get()
        .map_err(|e| format!("Failed to get HTTP client: {}", e))?;
    let client = HttpClientHolder::new(http_client);

    // 尝试下载校验和文件（带 spinner）
    spinner!("Verifying integrity...").with(|| -> Result<(), Box<dyn std::error::Error>> {
        match client.get(&checksum_url).send() {
            Ok(response) => {
                if response.status == 404 {
                    // 校验和文件不存在
                    let actual_hash = calculate_sha256(archive_path).ok();
                    log_info!(
                        "Checksum skipped (404) | url={} actual_hash={:?}",
                        checksum_url,
                        actual_hash
                    );
                    warning!("Checksum file not found, skipping integrity verification");
                    warning!("  Checksum URL: {}", checksum_url);
                    warning!("  This may indicate the release does not include checksum files");
                    warning!("  Proceeding with update without verification...");

                    if let Some(hash) = actual_hash {
                        info!("Downloaded file SHA256: {}", hash);
                    }
                    return Ok(());
                }

                if !response.is_success() {
                    let actual_hash = calculate_sha256(archive_path).ok();
                    log_info!(
                        "Checksum skipped (HTTP {}) | url={} actual_hash={:?}",
                        response.status,
                        checksum_url,
                        actual_hash
                    );
                    warning!("Failed to download checksum file: HTTP {}", response.status);
                    warning!("  Proceeding with update without verification...");

                    if let Some(hash) = actual_hash {
                        info!("Downloaded file SHA256: {}", hash);
                    }
                    return Ok(());
                }

                let checksum_content = response
                    .text()
                    .map_err(|e| format!("Failed to read checksum file: {}", e))?
                    .to_string();

                // 解析哈希值
                let expected_hash = parse_hash_from_content(&checksum_content)
                    .map_err(|e| format!("Failed to parse checksum file: {}", e))?;

                // 验证文件
                verify_checksum(archive_path, &expected_hash)
                    .map_err(|e| format!("File integrity verification failed: {}", e))?;

                log_info!(
                "Integrity verified | archive={} checksum_url={} expected_hash={} algorithm=sha256",
                archive_path.display(),
                checksum_url,
                expected_hash
            );
            }
            Err(e) => {
                // 网络错误等，给出警告但继续
                let actual_hash = calculate_sha256(archive_path).ok();
                log_info!(
                    "Checksum skipped (error) | url={} error={} actual_hash={:?}",
                    checksum_url,
                    e,
                    actual_hash
                );
                warning!("Failed to download checksum file: {}", e);
                warning!("  Proceeding with update without verification...");

                // 仍然计算并显示文件的 SHA256，供用户参考
                if let Some(hash) = actual_hash {
                    info!("Downloaded file SHA256: {}", hash);
                }
            }
        }

        Ok(())
    })?;

    Ok(())
}

/// 解压归档文件
///
/// 解压 tar.gz 或 zip 文件到指定目录。
pub fn extract_archive(
    archive_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    log_debug!("Extracting: {}", archive_path.display());
    log_debug!("Extracting to: {}", output_dir.display());

    let spinner = Spinner::new("Extracting...");
    let spinner_instance = spinner.start();

    let result = archive::extract(archive_path, output_dir);

    spinner_instance.stop();

    result.map_err(|e| format!("Failed to extract archive: {}", e))?;

    log_info!(
        "Archive extracted | archive={} output_dir={}",
        archive_path.display(),
        output_dir.display()
    );

    Ok(())
}
