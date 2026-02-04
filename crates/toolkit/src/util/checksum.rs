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

use sha2::{Digest, Sha256};
use thiserror::Error;

// ============================================================================
// 错误类型
// ============================================================================

/// 校验和相关错误
#[derive(Debug, Error)]
pub enum ChecksumError {
    /// I/O 错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// 无效的校验和文件格式
    #[error("Invalid checksum file format")]
    InvalidFormat,

    /// 校验和验证失败
    #[error("Checksum verification failed: expected {expected}, got {actual}")]
    VerificationFailed {
        /// 期望的哈希值
        expected: String,
        /// 实际的哈希值
        actual: String,
    },
}

// ============================================================================
// 验证结果
// ============================================================================

/// 校验和验证结果
#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// 是否验证通过
    pub verified: bool,
    /// 期望的哈希值
    pub expected: String,
    /// 实际的哈希值
    pub actual: String,
}

// ============================================================================
// 校验和函数
// ============================================================================

/// 计算文件的 SHA256 哈希值
///
/// 读取文件并计算其 SHA256 哈希值。使用流式读取，
/// 适合处理大文件。
///
/// # 参数
///
/// * `file_path` - 要计算哈希值的文件路径
///
/// # 返回
///
/// 返回文件的 SHA256 哈希值（十六进制字符串，小写）。
///
/// # 错误
///
/// - 文件不存在或无法打开
/// - 读取文件时发生 I/O 错误
///
/// # 示例
///
/// ```no_run
/// use std::path::Path;
/// use toolkit::util::checksum::calculate_sha256;
///
/// let hash = calculate_sha256(Path::new("file.tar.gz")).unwrap();
/// println!("SHA256: {}", hash);
/// ```
pub fn calculate_sha256(file_path: &Path) -> Result<String, ChecksumError> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
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
/// use toolkit::util::checksum::parse_hash_from_content;
///
/// // 标准格式：hash  filename
/// let hash = parse_hash_from_content("abc123def456  file.tar.gz").unwrap();
/// assert_eq!(hash, "abc123def456");
///
/// // 仅哈希值
/// let hash = parse_hash_from_content("abc123def456").unwrap();
/// assert_eq!(hash, "abc123def456");
/// ```
pub fn parse_hash_from_content(content: &str) -> Result<String, ChecksumError> {
    content
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().next().map(|s| s.to_string()))
        .ok_or(ChecksumError::InvalidFormat)
}

/// 验证文件完整性
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
/// 返回 `VerifyResult`，包含验证状态和哈希值信息。
///
/// # 错误
///
/// - 文件不存在或无法打开
/// - 读取文件时发生 I/O 错误
/// - 哈希值不匹配（返回 `ChecksumError::VerificationFailed`）
///
/// # 示例
///
/// ```no_run
/// use std::path::Path;
/// use toolkit::util::checksum::verify;
///
/// let result = verify(
///     Path::new("file.tar.gz"),
///     "abc123def456..."
/// ).unwrap();
///
/// if result.verified {
///     println!("File integrity verified!");
/// }
/// ```
pub fn verify(file_path: &Path, expected_hash: &str) -> Result<VerifyResult, ChecksumError> {
    let actual_hash = calculate_sha256(file_path)?;

    if actual_hash == expected_hash {
        Ok(VerifyResult {
            verified: true,
            expected: expected_hash.to_string(),
            actual: actual_hash,
        })
    } else {
        Err(ChecksumError::VerificationFailed {
            expected: expected_hash.to_string(),
            actual: actual_hash,
        })
    }
}

/// 验证文件完整性（非严格模式）
///
/// 与 `verify` 类似，但不匹配时不返回错误，而是返回 `VerifyResult`。
///
/// # 参数
///
/// * `file_path` - 要验证的文件路径
/// * `expected_hash` - 期望的 SHA256 哈希值
///
/// # 返回
///
/// 返回 `VerifyResult`，`verified` 字段指示是否匹配。
pub fn verify_lenient(
    file_path: &Path,
    expected_hash: &str,
) -> Result<VerifyResult, ChecksumError> {
    let actual_hash = calculate_sha256(file_path)?;
    let verified = actual_hash == expected_hash;

    Ok(VerifyResult {
        verified,
        expected: expected_hash.to_string(),
        actual: actual_hash,
    })
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
/// use toolkit::util::checksum::build_checksum_url;
///
/// let url = build_checksum_url("https://example.com/file.tar.gz");
/// assert_eq!(url, "https://example.com/file.tar.gz.sha256");
/// ```
pub fn build_checksum_url(download_url: &str) -> String {
    format!("{}.sha256", download_url)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_calculate_sha256() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();

        let hash = calculate_sha256(file.path()).unwrap();
        // SHA256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_parse_hash_from_content() {
        // 标准格式
        let hash = parse_hash_from_content("abc123  file.tar.gz").unwrap();
        assert_eq!(hash, "abc123");

        // 仅哈希值
        let hash = parse_hash_from_content("abc123").unwrap();
        assert_eq!(hash, "abc123");

        // 多行内容（取第一行）
        let hash = parse_hash_from_content("abc123  file.tar.gz\nother content").unwrap();
        assert_eq!(hash, "abc123");

        // 空内容
        let result = parse_hash_from_content("");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_success() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();

        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let result = verify(file.path(), expected).unwrap();
        assert!(result.verified);
    }

    #[test]
    fn test_verify_failure() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();

        let result = verify(file.path(), "wrong_hash");
        assert!(matches!(
            result,
            Err(ChecksumError::VerificationFailed { .. })
        ));
    }

    #[test]
    fn test_verify_lenient() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();

        // 正确的哈希
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let result = verify_lenient(file.path(), expected).unwrap();
        assert!(result.verified);

        // 错误的哈希（不报错，返回 verified = false）
        let result = verify_lenient(file.path(), "wrong_hash").unwrap();
        assert!(!result.verified);
    }

    #[test]
    fn test_build_checksum_url() {
        let url = build_checksum_url("https://example.com/file.tar.gz");
        assert_eq!(url, "https://example.com/file.tar.gz.sha256");
    }
}
