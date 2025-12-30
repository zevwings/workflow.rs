//! 校验和工具实现
//!
//! 提供文件校验和计算和验证功能。

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
    /// use workflow::base::checksum::Checksum;
    /// use workflow::log_message;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let hash = Checksum::calculate_file_sha256(Path::new("file.tar.gz"))?;
    /// log_message!("SHA256: {}", hash);
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
    /// use workflow::base::checksum::Checksum;
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
    /// use workflow::base::checksum::Checksum;
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
    /// use workflow::base::checksum::Checksum;
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

    /// 测试从内容解析哈希值（各种格式）
    ///
    /// ## 测试目的
    /// 验证 Checksum::parse_hash_from_content() 能够从各种格式的内容中解析哈希值。
    ///
    /// ## 测试场景
    /// 1. 测试标准格式：hash  filename
    /// 2. 测试只有哈希值的格式
    /// 3. 测试多行内容（只取第一行）
    /// 4. 测试带额外空格的格式
    ///
    /// ## 预期结果
    /// - 所有格式都能正确解析哈希值
    #[test]
    fn test_parse_hash_from_content_with_various_formats_parses_correctly_return_ok() -> Result<()>
    {
        // Arrange: 准备各种格式的内容

        // Act & Assert: 测试标准格式：hash  filename
        let content1 = "abc123def456789  file.tar.gz";
        let hash1 = Checksum::parse_hash_from_content(content1)?;
        assert_eq!(hash1, "abc123def456789");

        // Act & Assert: 测试只有哈希值的格式
        let content2 = "abc123def456789";
        let hash2 = Checksum::parse_hash_from_content(content2)?;
        assert_eq!(hash2, "abc123def456789");

        // Act & Assert: 测试多行内容（只取第一行）
        let content3 = "abc123def456789  file1.tar.gz\ndef456ghi789012  file2.tar.gz";
        let hash3 = Checksum::parse_hash_from_content(content3)?;
        assert_eq!(hash3, "abc123def456789");

        // Act & Assert: 测试带额外空格的格式
        let content4 = "  abc123def456789   file.tar.gz  ";
        let hash4 = Checksum::parse_hash_from_content(content4)?;
        assert_eq!(hash4, "abc123def456789");

        Ok(())
    }

    /// 测试从无效内容解析哈希值
    ///
    /// ## 测试目的
    /// 验证 Checksum::parse_hash_from_content() 对无效内容返回错误。
    ///
    /// ## 测试场景
    /// 1. 测试空内容
    /// 2. 测试只有空白字符的内容
    /// 3. 测试只有换行符的内容
    /// 4. 验证返回错误
    ///
    /// ## 预期结果
    /// - 所有无效内容都返回错误
    #[test]
    fn test_parse_hash_from_content_with_invalid_content_returns_error() {
        // Arrange: 准备无效内容

        // Act & Assert: 测试空内容
        let result1 = Checksum::parse_hash_from_content("");
        assert!(result1.is_err());

        // Act & Assert: 测试只有空白字符的内容
        let result2 = Checksum::parse_hash_from_content("   \n\t  ");
        assert!(result2.is_err());

        // Act & Assert: 测试只有换行符的内容
        let result3 = Checksum::parse_hash_from_content("\n\n");
        assert!(result3.is_err());
    }

    /// 测试构建 SHA256 URL（各种 URL 格式）
    ///
    /// ## 测试目的
    /// 验证 Checksum::build_url() 能够为各种 URL 格式构建 SHA256 URL。
    ///
    /// ## 测试场景
    /// 1. 测试基本 URL 构建
    /// 2. 测试带查询参数的 URL
    /// 3. 测试带锚点的 URL
    /// 4. 测试简单文件名
    /// 5. 测试空字符串
    ///
    /// ## 预期结果
    /// - 所有 URL 格式都能正确构建 SHA256 URL（在末尾添加 .sha256）
    #[test]
    fn test_build_url_with_various_urls_returns_sha256_url() {
        // Arrange: 准备各种URL格式

        // Act & Assert: 测试基本 URL 构建
        let url1 = "https://example.com/file.tar.gz";
        assert_eq!(
            Checksum::build_url(url1),
            "https://example.com/file.tar.gz.sha256"
        );

        // Act & Assert: 测试带查询参数的 URL
        let url2 = "https://example.com/file.tar.gz?version=1.0";
        assert_eq!(
            Checksum::build_url(url2),
            "https://example.com/file.tar.gz?version=1.0.sha256"
        );

        // Act & Assert: 测试带锚点的 URL
        let url3 = "https://example.com/file.tar.gz#section";
        assert_eq!(
            Checksum::build_url(url3),
            "https://example.com/file.tar.gz#section.sha256"
        );

        // Act & Assert: 测试简单文件名
        let url4 = "file.tar.gz";
        assert_eq!(Checksum::build_url(url4), "file.tar.gz.sha256");

        // Act & Assert: 测试空字符串
        let url5 = "";
        assert_eq!(Checksum::build_url(url5), ".sha256");
    }
}
