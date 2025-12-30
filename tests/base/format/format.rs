//! Base/Util 格式化工具测试
//!
//! 测试util模块中各种格式化和处理工具的核心业务逻辑，包括：
//! - 文件大小格式化算法
//! - 敏感信息掩码处理
//! - 日期时间格式化
//! - 校验和计算和验证
//! - 字符串处理工具

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::common::performance::measure_test_time_with_threshold;
use color_eyre::Result;
use rstest::rstest;
use std::time::Duration;

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::cli_env;
use workflow::base::checksum::Checksum;
use workflow::base::format::DisplayFormatter;
use workflow::base::format::{date::format_filename_timestamp, Sensitive};

// 注意：以下测试模块已迁移到 src/lib/base/format/display.rs
// - format_size_tests 模块（8个文件大小格式化测试）

// 注意：以下测试模块已迁移到 src/lib/base/format/sensitive.rs
// - sensitive_string_tests 模块（7个敏感信息掩码测试）

// 注意：以下测试模块已迁移到 src/lib/base/format/date.rs
// - date_format_tests 模块（7个日期时间格式化测试）

#[cfg(test)]
mod checksum_tests {
    use super::*;

    // ==================== 校验和计算测试 ====================

    /// 测试计算文件的SHA256哈希值
    ///
    /// ## 测试目的
    /// 验证 Checksum::calculate_file_sha256() 能够正确计算文件的SHA256哈希值。
    ///
    /// ## 测试场景
    /// 1. 创建测试文件并写入内容
    /// 2. 计算文件的SHA256哈希值
    /// 3. 验证哈希值格式和内容
    ///
    /// ## 预期结果
    /// - 哈希值长度为64个十六进制字符
    /// - 哈希值与预期值匹配
    #[rstest]
    fn test_calculate_file_sha256_return_ok(cli_env: CliTestEnv) -> Result<()> {
        let env = &cli_env;
        let file_path = env.path().join("test_file.txt");

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

    /// 测试计算空文件的SHA256哈希值
    ///
    /// ## 测试目的
    /// 验证 Checksum::calculate_file_sha256() 能够正确处理空文件。
    ///
    /// ## 预期结果
    /// - 空文件的SHA256哈希值为标准空文件哈希值
    /// - 哈希值格式正确
    #[rstest]
    fn test_calculate_empty_file_sha256_return_empty(cli_env: CliTestEnv) -> Result<()> {
        let env = &cli_env;
        let file_path = env.path().join("empty_file.txt");

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

    /// 测试计算大文件的SHA256哈希值
    ///
    /// ## 测试目的
    /// 验证 Checksum::calculate_file_sha256() 能够正确处理大文件（超过缓冲区大小）。
    ///
    /// ## 测试场景
    /// 创建10KB的测试文件并计算哈希值
    ///
    /// ## 预期结果
    /// - 大文件的哈希值计算成功
    /// - 哈希值格式正确（64个十六进制字符）
    #[rstest]
    fn test_calculate_large_file_sha256_return_ok(cli_env: CliTestEnv) -> Result<()> {
        let env = &cli_env;
        let file_path = env.path().join("large_file.txt");

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

    /// 测试从内容中解析哈希值
    ///
    /// ## 测试目的
    /// 验证 Checksum::parse_hash_from_content() 能够从各种格式的内容中解析哈希值。
    ///
    /// ## 测试场景
    /// 测试标准格式（hash filename）、只有哈希值、多行内容、带额外空格等格式
    ///
    /// ## 预期结果
    /// - 所有格式都能正确解析哈希值
    /// - 多行内容只取第一行
    #[test]
    fn test_parse_hash_from_content_return_ok() -> Result<()> {
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

    /// 测试从无效内容中解析哈希值
    ///
    /// ## 测试目的
    /// 验证 Checksum::parse_hash_from_content() 能够正确处理无效内容。
    ///
    /// ## 测试场景
    /// 测试空内容、只包含空格、只包含文件名等无效格式
    ///
    /// ## 预期结果
    /// - 无效内容返回错误
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

    /// 测试文件完整性验证（成功场景）
    ///
    /// ## 测试目的
    /// 验证 Checksum::verify() 能够正确验证文件完整性，当哈希值匹配时返回成功。
    ///
    /// ## 测试场景
    /// 1. 创建测试文件
    /// 2. 计算文件的SHA256哈希值
    /// 3. 使用正确的哈希值验证文件
    ///
    /// ## 预期结果
    /// - 验证成功（verified = true）
    /// - 消息包含验证通过的信息
    #[rstest]
    fn test_verify_success_return_true(cli_env: CliTestEnv) -> Result<()> {
        let env = &cli_env;
        let file_path = env.path().join("verify_test.txt");

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

    /// 测试文件完整性验证（失败场景）
    ///
    /// ## 测试目的
    /// 验证 Checksum::verify() 能够正确检测文件完整性验证失败。
    ///
    /// ## 测试场景
    /// 1. 创建测试文件
    /// 2. 使用错误的哈希值验证文件
    /// 3. 验证错误处理
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含预期和实际的哈希值
    #[rstest]
    fn test_verify_failure(cli_env: CliTestEnv) -> Result<()> {
        let env = &cli_env;
        let file_path = env.path().join("verify_fail_test.txt");

        // 创建测试文件
        let mut file = fs::File::create(&file_path)?;
        file.write_all(b"Test content")?;
        file.sync_all()?;
        drop(file);

        // 使用错误的哈希值进行验证
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = Checksum::verify(&file_path, wrong_hash);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("File integrity verification failed"));
        assert!(error_msg.contains("Expected:"));
        assert!(error_msg.contains("Actual:"));

        Ok(())
    }

    /// 测试构建下载URL
    ///
    /// ## 测试目的
    /// 验证 Checksum::build_url() 能够正确构建文件下载URL。
    ///
    /// ## 预期结果
    /// - URL格式正确
    #[test]
    fn test_build_url() {
        // 测试基本 URL 构建
        let url1 = "https://example.com/file.tar.gz";
        assert_eq!(
            Checksum::build_url(url1),
            "https://example.com/file.tar.gz.sha256"
        );

        // 测试带查询参数的 URL
        let url2 = "https://example.com/file.tar.gz?version=1.0";
        assert_eq!(
            Checksum::build_url(url2),
            "https://example.com/file.tar.gz?version=1.0.sha256"
        );

        // 测试带锚点的 URL
        let url3 = "https://example.com/file.tar.gz#section";
        assert_eq!(
            Checksum::build_url(url3),
            "https://example.com/file.tar.gz#section.sha256"
        );

        // 测试简单文件名
        let url4 = "file.tar.gz";
        assert_eq!(Checksum::build_url(url4), "file.tar.gz.sha256");

        // 测试空字符串
        let url5 = "";
        assert_eq!(Checksum::build_url(url5), ".sha256");
    }

    /// 测试文件不存在时的错误处理
    ///
    /// ## 测试目的
    /// 验证 Checksum::calculate_file_sha256() 能够正确处理文件不存在的情况。
    ///
    /// ## 预期结果
    /// - 返回错误
    /// - 错误消息包含 "Failed to open file"
    #[test]
    fn test_file_not_found() {
        let non_existent_path = Path::new("/this/path/does/not/exist/file.txt");
        let result = Checksum::calculate_file_sha256(non_existent_path);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to open file"));
    }

    /// 测试已知内容的哈希值计算
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Checksum::calculate_file_sha256() 能够计算已知内容的正确哈希值。
    ///
    /// ## 测试场景
    /// 测试多种已知内容的SHA256哈希值（标准测试向量）
    ///
    /// ## 预期结果
    /// - 所有已知内容的哈希值与预期值完全匹配
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
    fn test_known_hash_values_return_ok(
        cli_env: CliTestEnv,
        #[case] content: &str,
        #[case] expected_hash: &str,
    ) -> Result<()> {
        let env = &cli_env;
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
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // ==================== 集成测试 ====================

    /// 测试格式化工具的集成使用
    ///
    /// ## 测试目的
    /// 验证各种格式化工具（文件大小、时间戳、敏感信息掩码）能够协同工作。
    ///
    /// ## 测试场景
    /// 1. 格式化文件大小
    /// 2. 生成文件名时间戳
    /// 3. 掩码API密钥
    /// 4. 组合使用生成报告文件名
    ///
    /// ## 预期结果
    /// - 所有格式化工具正常工作
    /// - 组合使用生成正确的报告文件名
    #[test]
    fn test_format_utilities_integration() -> Result<()> {
        // 测试各种格式化工具的集成使用
        let file_size = DisplayFormatter::size(1024 * 1024 * 5); // 5MB
        let timestamp = format_filename_timestamp();
        let masked_key = "very_long_api_key_123456789".mask();

        // 验证格式化结果
        assert_eq!(file_size, "5.00 MB");
        let timestamp_regex =
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$").map_err(|e| {
                color_eyre::eyre::eyre!("Filename timestamp regex should be valid: {}", e)
            })?;
        assert!(timestamp_regex.is_match(&timestamp));
        assert_eq!(masked_key, "very***6789");

        // 模拟生成报告文件名
        let report_filename = format!(
            "DOWNLOAD_REPORT_{}_{}.md",
            timestamp,
            file_size.replace(" ", "_")
        );
        assert!(report_filename.contains("DOWNLOAD_REPORT_"));
        assert!(report_filename.contains("5.00_MB"));
        assert!(report_filename.ends_with(".md"));
        Ok(())
    }

    /// 测试校验和和格式化工具的集成使用
    ///
    /// ## 测试目的
    /// 验证校验和计算和格式化工具能够协同工作。
    ///
    /// ## 测试场景
    /// 1. 创建测试文件
    /// 2. 计算文件哈希值
    /// 3. 格式化文件大小
    /// 4. 组合使用生成报告
    ///
    /// ## 预期结果
    /// - 校验和计算成功
    /// - 格式化工具正常工作
    /// - 集成使用无错误
    #[rstest]
    fn test_checksum_and_format_integration_return_ok(cli_env: CliTestEnv) -> Result<()> {
        let env = &cli_env;
        let file_path = env.path().join("integration_test.txt");

        // 创建测试文件
        let content = "Integration test content";
        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);

        // 计算文件大小和哈希值
        let file_metadata = fs::metadata(&file_path)?;
        let file_size = DisplayFormatter::size(file_metadata.len());
        let hash = Checksum::calculate_file_sha256(&file_path)?;
        let masked_hash = hash.mask();

        // 验证结果
        assert_eq!(file_size, format!("{} B", content.len()));
        assert_eq!(hash.len(), 64);
        assert_eq!(
            masked_hash,
            format!("{}***{}", &hash[..4], &hash[hash.len() - 4..])
        );

        // 验证文件完整性
        let verify_result = Checksum::verify(&file_path, &hash)?;
        assert!(verify_result.verified);

        Ok(())
    }

    /// 测试错误处理的一致性
    ///
    /// ## 测试目的
    /// 验证各个模块的错误处理保持一致，错误消息包含有用信息。
    ///
    /// ## 测试场景
    /// 1. 测试文件不存在时的错误处理
    /// 2. 测试无效内容解析时的错误处理
    /// 3. 验证错误消息格式
    ///
    /// ## 预期结果
    /// - 所有错误情况都能正确返回错误
    /// - 错误消息包含有用的信息（如文件路径、错误类型等）
    #[test]
    fn test_error_handling_consistency() {
        // 测试各个模块的错误处理一致性

        // 测试文件不存在的情况
        let non_existent = Path::new("/does/not/exist");
        let checksum_result = Checksum::calculate_file_sha256(non_existent);
        assert!(checksum_result.is_err());

        // 测试无效内容解析
        let parse_result = Checksum::parse_hash_from_content("");
        assert!(parse_result.is_err());

        // 验证错误消息包含有用信息
        let error_msg = checksum_result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to open file") || error_msg.contains("No such file"));
    }

    /// 测试格式化函数的性能特征
    ///
    /// ## 测试目的
    /// 验证格式化函数（文件大小格式化、敏感信息掩码）的性能表现。
    ///
    /// ## 测试场景
    /// 执行1000次格式化操作，测量总耗时
    ///
    /// ## 预期结果
    /// - 1000次格式化操作应在100毫秒内完成
    /// - 性能表现良好
    #[test]
    fn test_performance_characteristics_return_ok() -> Result<()> {
        // 测试格式化函数的性能特征（应该很快）
        // 1000次格式化操作应该在很短时间内完成（< 100ms）
        measure_test_time_with_threshold(
            "test_performance_characteristics_return_ok",
            Duration::from_millis(100),
            || {
                for i in 0..1000 {
                    let _ = DisplayFormatter::size(i * 1024);
                    let _ = format!("key_{}", i).mask();
                }
                Ok(())
            },
        )
    }
}
