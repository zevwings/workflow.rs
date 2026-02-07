//! 下载和解压模块
//!
//! 提供文件下载和解压功能。

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use prompt::{info, success, warning, Progress, Spinner};
use toolkit::{
    archive, build_checksum_url, calculate_sha256, parse_hash_from_content, verify_checksum,
    SizeExt,
};

use http::HttpClient;

use super::types::{GITHUB_DOWNLOAD_BASE, REPO_NAME, REPO_OWNER};

/// 构建下载 URL
///
/// 根据平台和版本号拼接下载链接。
pub fn build_download_url(version: &str, platform: &str) -> String {
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
pub fn download_file(url: &str, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("Downloading update package...");
    toolkit::log_debug!("Download URL: {}", url);
    toolkit::log_debug!("Saving to: {}", output_path.display());

    // 如果文件已存在且不完整，先删除它
    if output_path.exists() {
        if let Err(e) = fs::remove_file(output_path) {
            toolkit::log_debug!("Failed to delete incomplete file: {}", e);
        }
    }

    // 使用流式下载
    let http_client = HttpClient::global()?;
    let mut response = http_client
        .get(url)
        .stream()
        .map_err(|e| format!("Failed to send HTTP request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()).into());
    }

    // 获取文件总大小（如果可用）
    let total_size: Option<u64> = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    // 创建进度条
    let progress = if let Some(size) = total_size {
        info!("File size: {}", size.to_size_string());
        Progress::new_download(size, "Downloading update package...")
    } else {
        Progress::new_unknown("Downloading update package...")
    };

    let mut file = File::create(output_path)
        .map_err(|e| format!("Failed to create file {}: {}", output_path.display(), e))?;

    let mut buffer = vec![0u8; 8192];
    let mut downloaded_bytes = 0u64;

    loop {
        let bytes_read = response
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read response data: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        downloaded_bytes += bytes_read as u64;
        progress.set_position(downloaded_bytes);
    }

    progress.finish_with_message("Download complete!");
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

    toolkit::log_debug!("Checksum URL: {}", checksum_url);

    let http_client = HttpClient::global()?;

    // 尝试下载校验和文件
    match http_client.get(&checksum_url).send() {
        Ok(response) => {
            if response.status == 404 {
                // 校验和文件不存在
                warning!("Checksum file not found, skipping integrity verification");
                warning!("  Checksum URL: {}", checksum_url);
                warning!("  This may indicate the release does not include checksum files");
                warning!("  Proceeding with update without verification...");

                // 仍然计算并显示文件的 SHA256，供用户参考
                if let Ok(actual_hash) = calculate_sha256(archive_path) {
                    info!("Downloaded file SHA256: {}", actual_hash);
                }
                return Ok(());
            }

            if response.status < 200 || response.status >= 300 {
                warning!("Failed to download checksum file: HTTP {}", response.status);
                warning!("  Proceeding with update without verification...");

                if let Ok(actual_hash) = calculate_sha256(archive_path) {
                    info!("Downloaded file SHA256: {}", actual_hash);
                }
                return Ok(());
            }

            let checksum_content = response
                .into_text()
                .map_err(|e| format!("Failed to read checksum file: {}", e))?;

            // 解析哈希值
            let expected_hash = parse_hash_from_content(&checksum_content)
                .map_err(|e| format!("Failed to parse checksum file: {}", e))?;

            // 验证文件
            info!("Verifying file integrity...");
            verify_checksum(archive_path, &expected_hash)
                .map_err(|e| format!("File integrity verification failed: {}", e))?;

            success!("File integrity verification passed");
        }
        Err(e) => {
            // 网络错误等，给出警告但继续
            warning!("Failed to download checksum file: {}", e);
            warning!("  Proceeding with update without verification...");

            // 仍然计算并显示文件的 SHA256，供用户参考
            if let Ok(actual_hash) = calculate_sha256(archive_path) {
                info!("Downloaded file SHA256: {}", actual_hash);
            }
        }
    }

    Ok(())
}

/// 解压归档文件
///
/// 解压 tar.gz 或 zip 文件到指定目录。
pub fn extract_archive(
    archive_path: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    toolkit::log_debug!("Extracting: {}", archive_path.display());
    toolkit::log_debug!("Extracting to: {}", output_dir.display());

    let spinner = Spinner::new("Extracting update package...");
    let spinner_instance = spinner.start();

    let result = archive::extract(archive_path, output_dir);

    spinner_instance.stop();

    result.map_err(|e| format!("Failed to extract archive: {}", e))?;

    success!("Extraction complete");

    Ok(())
}
