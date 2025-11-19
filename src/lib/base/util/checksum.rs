//! 校验和工具模块
//!
//! 本模块提供了文件校验和计算和验证功能，包括：
//! - 计算文件的 SHA256 哈希值
//! - 解析校验和文件内容
//! - 验证文件完整性
//! - 构建校验和 URL（纯字符串操作）

use crate::{log_info, log_success};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 校验和工具
///
/// 提供文件校验和计算和验证功能。
pub struct Checksum;

impl Checksum {
    /// 计算文件的 SHA256 哈希值
    ///
    /// 读取文件并计算其 SHA256 哈希值。
    ///
    /// # 参数
    ///
    /// * `file_path` - 要计算哈希值的文件路径
    ///
    /// # 返回
    ///
    /// 返回文件的 SHA256 哈希值（十六进制字符串）。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::util::checksum::Checksum;
    /// use std::path::Path;
    ///
    /// let hash = Checksum::calculate_file_sha256(Path::new("file.tar.gz"))?;
    /// println!("SHA256: {}", hash);
    /// ```
    pub fn calculate_file_sha256(file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .context("Failed to read file for checksum calculation")?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// 从校验和文件内容中提取哈希值
    ///
    /// 解析校验和文件内容（通常是 "hash  filename" 格式或只有 "hash"），
    /// 提取并返回哈希值。
    ///
    /// # 参数
    ///
    /// * `content` - 校验和文件的文本内容
    ///
    /// # 返回
    ///
    /// 返回提取的哈希值。如果内容格式无效，返回错误。
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::util::checksum::Checksum;
    ///
    /// let content = "abc123def456  file.tar.gz";
    /// let hash = Checksum::parse_hash_from_content(content)?;
    /// assert_eq!(hash, "abc123def456");
    /// ```
    pub fn parse_hash_from_content(content: &str) -> Result<String> {
        content
            .lines()
            .next()
            .and_then(|line| {
                // 提取 SHA256 哈希值（格式可能是 "hash  filename" 或只有 "hash"）
                line.split_whitespace().next().map(|s| s.to_string())
            })
            .ok_or_else(|| anyhow::anyhow!("Invalid checksum file format"))
    }

    /// 验证文件完整性（通过比较哈希值）
    ///
    /// 计算文件的 SHA256 哈希值，并与期望的哈希值进行比较。
    ///
    /// # 参数
    ///
    /// * `file_path` - 要验证的文件路径
    /// * `expected_hash` - 期望的 SHA256 哈希值
    ///
    /// # 返回
    ///
    /// 如果哈希值匹配，返回 `Ok(())`。
    /// 如果哈希值不匹配，返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::util::checksum::Checksum;
    /// use std::path::Path;
    ///
    /// let file_path = Path::new("file.tar.gz");
    /// let expected_hash = "abc123def456...";
    /// Checksum::verify(file_path, expected_hash)?;
    /// ```
    pub fn verify(file_path: &Path, expected_hash: &str) -> Result<()> {
        log_info!("正在验证文件完整性...");

        let actual_hash = Self::calculate_file_sha256(file_path)?;

        if actual_hash == expected_hash {
            log_success!("  文件完整性验证通过");
            Ok(())
        } else {
            anyhow::bail!(
                "文件完整性验证失败！\n  期望: {}\n  实际: {}",
                expected_hash,
                actual_hash
            );
        }
    }

    /// 从下载 URL 构建校验和 URL
    ///
    /// 在下载 URL 后添加 `.sha256` 后缀来构建校验和文件的 URL。
    ///
    /// # 参数
    ///
    /// * `download_url` - 下载文件的 URL
    ///
    /// # 返回
    ///
    /// 返回校验和文件的 URL。
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::util::checksum::Checksum;
    ///
    /// let url = "https://example.com/file.tar.gz";
    /// assert_eq!(Checksum::build_url(url), "https://example.com/file.tar.gz.sha256");
    /// ```
    pub fn build_url(url: &str) -> String {
        format!("{}.sha256", url)
    }
}
