//! Checksum 模块测试
//!
//! 测试校验和工具的核心功能，包括 SHA256 哈希计算、解析和验证。

use std::fs;
use std::io::Write;
use std::path::Path;

use color_eyre::Result;
use rstest::rstest;
use tempfile::tempdir;

use workflow::base::checksum::Checksum;

// ==================== 校验和计算测试 ====================

#[test]
fn test_calculate_file_sha256_with_valid_file_returns_hash() -> Result<()> {
    // Arrange: 准备测试文件
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test_file.txt");
    let mut file = fs::File::create(&file_path)?;
    file.write_all(b"Hello, World!")?;
    file.sync_all()?;
    drop(file);

    // Act: 计算哈希值
    let hash = Checksum::calculate_file_sha256(&file_path)?;

    // Assert: 验证哈希值格式和内容正确
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    let expected_hash = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
    assert_eq!(hash, expected_hash);

    Ok(())
}

#[test]
fn test_calculate_file_sha256_with_empty_file_returns_empty_hash() -> Result<()> {
    // Arrange: 准备空文件
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("empty_file.txt");
    fs::File::create(&file_path)?;

    // Act: 计算空文件的哈希值
    let hash = Checksum::calculate_file_sha256(&file_path)?;

    // Assert: 验证空文件的SHA256哈希值
    let expected_empty_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    assert_eq!(hash, expected_empty_hash);

    Ok(())
}

#[test]
fn test_calculate_file_sha256_with_large_file_returns_hash() -> Result<()> {
    // Arrange: 准备较大的测试文件（超过缓冲区大小）
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("large_file.txt");
    let mut file = fs::File::create(&file_path)?;
    let data = "A".repeat(10000); // 10KB 数据
    file.write_all(data.as_bytes())?;
    file.sync_all()?;
    drop(file);

    // Act: 计算哈希值
    let hash = Checksum::calculate_file_sha256(&file_path)?;

    // Assert: 验证哈希值格式正确
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    Ok(())
}

#[test]
fn test_parse_hash_from_content_with_various_formats_parses_correctly() -> Result<()> {
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

#[test]
fn test_verify_with_correct_hash_returns_success() -> Result<()> {
    // Arrange: 准备测试文件和正确的哈希值
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("verify_test.txt");
    let mut file = fs::File::create(&file_path)?;
    file.write_all(b"Test content for verification")?;
    file.sync_all()?;
    drop(file);
    let actual_hash = Checksum::calculate_file_sha256(&file_path)?;

    // Act: 验证文件（使用正确的哈希值）
    let result = Checksum::verify(&file_path, &actual_hash)?;

    // Assert: 验证验证成功且消息正确
    assert!(result.verified);
    assert_eq!(result.messages.len(), 2);
    assert!(result.messages[0].contains("Verifying file integrity"));
    assert!(result.messages[1].contains("verification passed"));

    Ok(())
}

#[test]
fn test_verify_with_incorrect_hash_returns_error() -> Result<()> {
    // Arrange: 准备测试文件和错误的哈希值
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("verify_fail_test.txt");
    let mut file = fs::File::create(&file_path)?;
    file.write_all(b"Test content")?;
    file.sync_all()?;
    drop(file);
    let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";

    // Act: 使用错误的哈希值进行验证
    let result = Checksum::verify(&file_path, wrong_hash);

    // Assert: 验证返回错误且错误消息包含相关信息
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("File integrity verification failed"));
    assert!(error_msg.contains("Expected:"));
    assert!(error_msg.contains("Actual:"));

    Ok(())
}

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

#[test]
fn test_calculate_file_sha256_with_nonexistent_file_returns_error() {
    // Arrange: 准备不存在的文件路径

    // Act: 尝试计算不存在文件的哈希值
    let non_existent_path = Path::new("/this/path/does/not/exist/file.txt");
    let result = Checksum::calculate_file_sha256(non_existent_path);

    // Assert: 验证返回错误且错误消息包含"Failed to open file"
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to open file"));
}

#[rstest]
#[case(
    "Hello, World!",
    "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
)]
#[case("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")]
#[case(
    "a",
    "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"
)]
#[case(
    "abc",
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
)]
fn test_known_hash_values(#[case] content: &str, #[case] expected_hash: &str) -> Result<()> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("hash_test.txt");

    // 创建测试文件
    let mut file = fs::File::create(&file_path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    drop(file);

    // 计算哈希值并验证
    let hash = Checksum::calculate_file_sha256(&file_path)?;
    assert_eq!(hash, expected_hash);

    Ok(())
}
