//! Checksum 模块测试
//!
//! 测试校验和工具的核心功能，包括 SHA256 哈希计算、解析和验证。

use std::fs;
use std::io::Write;
use std::path::Path;

use color_eyre::Result;
use rstest::rstest;

use workflow::base::checksum::Checksum;
use crate::common::environments::CliTestEnv;

// ==================== Checksum Calculation Tests ====================

/// 测试计算文件 SHA256 哈希值（有效文件）
///
/// ## 测试目的
/// 验证 Checksum::calculate_file_sha256() 能够计算有效文件的 SHA256 哈希值。
///
/// ## 测试场景
/// 1. 创建测试文件并写入内容
/// 2. 计算文件的 SHA256 哈希值
/// 3. 验证哈希值格式和内容正确
///
/// ## 预期结果
/// - 返回64字符的十六进制哈希值，与预期值匹配
#[test]
fn test_calculate_file_sha256_with_valid_file_returns_hash() -> Result<()> {
    // Arrange: 准备测试文件
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("test_file.txt");
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

/// 测试计算文件 SHA256 哈希值（空文件）
///
/// ## 测试目的
/// 验证 Checksum::calculate_file_sha256() 能够计算空文件的 SHA256 哈希值。
///
/// ## 测试场景
/// 1. 创建空文件
/// 2. 计算空文件的 SHA256 哈希值
/// 3. 验证返回空文件的 SHA256 哈希值
///
/// ## 预期结果
/// - 返回空文件的 SHA256 哈希值（e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855）
#[test]
fn test_calculate_file_sha256_with_empty_file_returns_empty_hash() -> Result<()> {
    // Arrange: 准备空文件
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("empty_file.txt");
    fs::File::create(&file_path)?;

    // Act: 计算空文件的哈希值
    let hash = Checksum::calculate_file_sha256(&file_path)?;

    // Assert: 验证空文件的SHA256哈希值
    let expected_empty_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    assert_eq!(hash, expected_empty_hash);

    Ok(())
}

/// 测试计算文件 SHA256 哈希值（大文件）
///
/// ## 测试目的
/// 验证 Checksum::calculate_file_sha256() 能够计算大文件的 SHA256 哈希值。
///
/// ## 测试场景
/// 1. 创建较大的测试文件（超过缓冲区大小）
/// 2. 计算文件的 SHA256 哈希值
/// 3. 验证哈希值格式正确
///
/// ## 预期结果
/// - 返回64字符的十六进制哈希值
#[test]
fn test_calculate_file_sha256_with_large_file_returns_hash() -> Result<()> {
    // Arrange: 准备较大的测试文件（超过缓冲区大小）
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("large_file.txt");
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

/// 测试验证文件完整性（正确的哈希值）
///
/// ## 测试目的
/// 验证 Checksum::verify() 能够使用正确的哈希值验证文件完整性。
///
/// ## 测试场景
/// 1. 创建测试文件并计算哈希值
/// 2. 使用正确的哈希值验证文件
/// 3. 验证验证成功且消息正确
///
/// ## 预期结果
/// - 验证成功，消息包含 "verification passed"
#[test]
fn test_verify_with_correct_hash_returns_success() -> Result<()> {
    // Arrange: 准备测试文件和正确的哈希值
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("verify_test.txt");
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

/// 测试验证文件完整性（错误的哈希值）
///
/// ## 测试目的
/// 验证 Checksum::verify() 对错误的哈希值返回错误。
///
/// ## 测试场景
/// 1. 创建测试文件
/// 2. 使用错误的哈希值进行验证
/// 3. 验证返回错误且错误消息包含相关信息
///
/// ## 预期结果
/// - 返回错误，错误消息包含 "File integrity verification failed"、"Expected:"、"Actual:"
#[test]
fn test_verify_with_incorrect_hash_returns_error() -> Result<()> {
    // Arrange: 准备测试文件和错误的哈希值
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("verify_fail_test.txt");
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

/// 测试计算文件 SHA256 哈希值（不存在的文件）
///
/// ## 测试目的
/// 验证 Checksum::calculate_file_sha256() 对不存在的文件返回错误。
///
/// ## 测试场景
/// 1. 准备不存在的文件路径
/// 2. 尝试计算不存在文件的哈希值
/// 3. 验证返回错误且错误消息包含 "Failed to open file"
///
/// ## 预期结果
/// - 返回错误，错误消息包含 "Failed to open file"
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

/// 测试已知哈希值（参数化）
///
/// ## 测试目的
/// 使用参数化测试验证 Checksum::calculate_file_sha256() 对已知内容的哈希值计算正确。
///
/// ## 测试场景
/// 1. 创建包含已知内容的测试文件
/// 2. 计算文件的 SHA256 哈希值
/// 3. 验证哈希值与预期值匹配
///
/// ## 预期结果
/// - 所有已知内容的哈希值都与预期值匹配
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
    let env = CliTestEnv::new()?;
    let file_path = env.path().join("hash_test.txt");

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
