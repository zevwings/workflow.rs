//! 校验和工具模块
//!
//! 本模块提供了文件校验和计算和验证功能，包括：
//! - 计算文件的 SHA256 哈希值
//! - 解析校验和文件内容
//! - 验证文件完整性
//! - 构建校验和 URL（纯字符串操作）

use std::fs::File;
use std::io::Read;
use std::path::Path;

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use sha2::{Digest, Sha256};

/// 校验和验证结果
#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// 是否验证通过
    pub verified: bool,
    /// 消息列表
    pub messages: Vec<String>,
}

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
    /// use workflow::info;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let hash = Checksum::calculate_file_sha256(Path::new("file.tar.gz"))?;
    /// info!("SHA256: {}", hash);
    /// # Ok(())
    /// # }
    /// ```
    pub fn calculate_file_sha256(file_path: &Path) -> Result<String> {
        // 注意：这里直接使用 File::open() 进行流式读取，使用自定义缓冲区分块读取
        // 不需要 BufReader，因为代码已经手动管理了固定大小的缓冲区（8192 字节）
        let mut file = File::open(file_path)
            .wrap_err_with(|| format!("Failed to open file: {}", file_path.display()))?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .wrap_err("Failed to read file for checksum calculation")?;

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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let content = "abc123def456  file.tar.gz";
    /// let hash = Checksum::parse_hash_from_content(content)?;
    /// assert_eq!(hash, "abc123def456");
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_hash_from_content(content: &str) -> Result<String> {
        content
            .lines()
            .next()
            .and_then(|line| {
                // 提取 SHA256 哈希值（格式可能是 "hash  filename" 或只有 "hash"）
                line.split_whitespace().next().map(|s| s.to_string())
            })
            .ok_or_else(|| eyre!("Invalid checksum file format"))
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
    /// 返回 `VerifyResult`，包含验证状态和消息。
    /// 如果哈希值不匹配，返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::util::checksum::Checksum;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file_path = Path::new("file.tar.gz");
    /// let expected_hash = "abc123def456...";
    /// let result = Checksum::verify(file_path, expected_hash)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify(file_path: &Path, expected_hash: &str) -> Result<VerifyResult> {
        let actual_hash = Self::calculate_file_sha256(file_path)?;

        if actual_hash == expected_hash {
            Ok(VerifyResult {
                verified: true,
                messages: vec![
                    "Verifying file integrity...".to_string(),
                    "  File integrity verification passed".to_string(),
                ],
            })
        } else {
            color_eyre::eyre::bail!(
                "File integrity verification failed!\n  Expected: {}\n  Actual: {}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_file_sha256() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file.txt");

        // 创建测试文件
        let mut file = fs::File::create(&file_path)?;
        file.write_all(b"Hello, World!")?;
        file.sync_all()?;
        drop(file);

        // 计算哈希值
        let hash = Checksum::calculate_file_sha256(&file_path)?;

        // 验证哈希值格式（64个十六进制字符）
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // 验证具体的哈希值（"Hello, World!" 的 SHA256）
        let expected_hash = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
        assert_eq!(hash, expected_hash);

        Ok(())
    }

    #[test]
    fn test_calculate_empty_file_sha256() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("empty_file.txt");

        // 创建空文件
        fs::File::create(&file_path)?;

        // 计算空文件的哈希值
        let hash = Checksum::calculate_file_sha256(&file_path)?;

        // 空文件的 SHA256 哈希值
        let expected_empty_hash =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash, expected_empty_hash);

        Ok(())
    }

    #[test]
    fn test_calculate_large_file_sha256() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("large_file.txt");

        // 创建较大的测试文件（超过缓冲区大小）
        let mut file = fs::File::create(&file_path)?;
        let data = "A".repeat(10000); // 10KB 数据
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        drop(file);

        // 计算哈希值
        let hash = Checksum::calculate_file_sha256(&file_path)?;

        // 验证哈希值格式
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        Ok(())
    }

    #[test]
    fn test_parse_hash_from_content() -> Result<()> {
        // 测试标准格式：hash  filename
        let content1 = "abc123def456789  file.tar.gz";
        let hash1 = Checksum::parse_hash_from_content(content1)?;
        assert_eq!(hash1, "abc123def456789");

        // 测试只有哈希值的格式
        let content2 = "abc123def456789";
        let hash2 = Checksum::parse_hash_from_content(content2)?;
        assert_eq!(hash2, "abc123def456789");

        // 测试多行内容（只取第一行）
        let content3 = "abc123def456789  file1.tar.gz\ndef456ghi789012  file2.tar.gz";
        let hash3 = Checksum::parse_hash_from_content(content3)?;
        assert_eq!(hash3, "abc123def456789");

        // 测试带额外空格的格式
        let content4 = "  abc123def456789   file.tar.gz  ";
        let hash4 = Checksum::parse_hash_from_content(content4)?;
        assert_eq!(hash4, "abc123def456789");

        Ok(())
    }

    #[test]
    fn test_parse_hash_from_invalid_content() {
        // 测试空内容
        let result1 = Checksum::parse_hash_from_content("");
        assert!(result1.is_err());

        // 测试只有空白字符的内容
        let result2 = Checksum::parse_hash_from_content("   \n\t  ");
        assert!(result2.is_err());

        // 测试只有换行符的内容
        let result3 = Checksum::parse_hash_from_content("\n\n");
        assert!(result3.is_err());
    }

    #[test]
    fn test_verify_success() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("verify_test.txt");

        // 创建测试文件
        let mut file = fs::File::create(&file_path)?;
        file.write_all(b"Test content for verification")?;
        file.sync_all()?;
        drop(file);

        // 计算实际哈希值
        let actual_hash = Checksum::calculate_file_sha256(&file_path)?;

        // 验证文件（使用正确的哈希值）
        let result = Checksum::verify(&file_path, &actual_hash)?;

        assert!(result.verified);
        assert_eq!(result.messages.len(), 2);
        assert!(result.messages[0].contains("Verifying file integrity"));
        assert!(result.messages[1].contains("verification passed"));

        Ok(())
    }

    #[test]
    fn test_verify_failure() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("verify_fail_test.txt");

        // 创建测试文件
        let mut file = fs::File::create(&file_path)?;
        file.write_all(b"Test content")?;
        file.sync_all()?;
        drop(file);

        // 使用错误的哈希值进行验证
        let wrong_hash =
            "wrong_hash_value_123456789012345678901234567890123456789012345678901234567890";
        let result = Checksum::verify(&file_path, wrong_hash);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("verification failed"));
        assert!(error_msg.contains("Expected"));
        assert!(error_msg.contains("Actual"));

        Ok(())
    }

    #[test]
    fn test_build_url() {
        let url = "https://example.com/file.tar.gz";
        let checksum_url = Checksum::build_url(url);
        assert_eq!(checksum_url, "https://example.com/file.tar.gz.sha256");
    }

    #[test]
    fn test_build_url_with_query() {
        let url = "https://example.com/file.tar.gz?version=1.0";
        let checksum_url = Checksum::build_url(url);
        assert_eq!(
            checksum_url,
            "https://example.com/file.tar.gz?version=1.0.sha256"
        );
    }
}
