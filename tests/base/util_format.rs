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

use color_eyre::Result;
use rstest::rstest;
use tempfile::tempdir;

use workflow::util::format::{mask_sensitive_value, DisplayFormatter, Sensitive};
use workflow::util::{
    checksum::Checksum,
    date::{
        format_document_timestamp, format_filename_timestamp, format_last_updated,
        format_last_updated_with_time, DateFormat, Timezone,
    },
};

#[cfg(test)]
mod format_size_tests {
    use super::*;

    // ==================== 文件大小格式化测试 ====================

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(DisplayFormatter::size(0), "0 B");
        assert_eq!(DisplayFormatter::size(1), "1 B");
        assert_eq!(DisplayFormatter::size(512), "512 B");
        assert_eq!(DisplayFormatter::size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
        assert_eq!(DisplayFormatter::size(1536), "1.50 KB"); // 1024 + 512
        assert_eq!(DisplayFormatter::size(2048), "2.00 KB");
        assert_eq!(DisplayFormatter::size(1024 * 1023), "1023.00 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(DisplayFormatter::size(1024 * 1024), "1.00 MB");
        assert_eq!(DisplayFormatter::size(1024 * 1024 + 512 * 1024), "1.50 MB");
        assert_eq!(DisplayFormatter::size(1024 * 1024 * 5), "5.00 MB");
        assert_eq!(DisplayFormatter::size(1024 * 1024 * 1023), "1023.00 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(DisplayFormatter::size(1024_u64.pow(3)), "1.00 GB");
        assert_eq!(
            DisplayFormatter::size(1024_u64.pow(3) + 512 * 1024_u64.pow(2)),
            "1.50 GB"
        );
        assert_eq!(DisplayFormatter::size(1024_u64.pow(3) * 10), "10.00 GB");
    }

    #[test]
    fn test_format_size_terabytes() {
        assert_eq!(DisplayFormatter::size(1024_u64.pow(4)), "1.00 TB");
        assert_eq!(DisplayFormatter::size(1024_u64.pow(4) * 2), "2.00 TB");
        assert_eq!(
            DisplayFormatter::size(1024_u64.pow(4) + 512 * 1024_u64.pow(3)),
            "1.50 TB"
        );
    }

    #[rstest]
    #[case(0, "0 B")]
    #[case(1, "1 B")]
    #[case(1023, "1023 B")]
    #[case(1024, "1.00 KB")]
    #[case(1536, "1.50 KB")]
    #[case(1048576, "1.00 MB")] // 1024^2
    #[case(1073741824, "1.00 GB")] // 1024^3
    #[case(1099511627776, "1.00 TB")] // 1024^4
    #[case(2147483648, "2.00 GB")] // 2 * 1024^3
    #[case(5368709120, "5.00 GB")] // 5 * 1024^3
    fn test_format_size_parametrized(#[case] bytes: u64, #[case] expected: &str) {
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    #[test]
    fn test_format_size_precision() {
        // 测试小数精度
        assert_eq!(DisplayFormatter::size(1024 + 256), "1.25 KB"); // 1.25 KB
        assert_eq!(DisplayFormatter::size(1024 + 102), "1.10 KB"); // 约1.10 KB
        assert_eq!(DisplayFormatter::size(1024 + 51), "1.05 KB"); // 约1.05 KB
    }

    #[test]
    fn test_format_size_edge_cases() {
        // 测试边界值
        assert_eq!(
            DisplayFormatter::size(u64::MAX),
            format!("{:.2} TB", u64::MAX as f64 / 1024_f64.powi(4))
        );

        // 测试刚好达到下一个单位的值
        assert_eq!(DisplayFormatter::size(1024 - 1), "1023 B");
        assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
        assert_eq!(DisplayFormatter::size(1024 * 1024 - 1), "1024.00 KB");
        assert_eq!(DisplayFormatter::size(1024 * 1024), "1.00 MB");
    }
}

#[cfg(test)]
mod sensitive_string_tests {
    use super::*;

    // ==================== 敏感信息掩码测试 ====================

    #[test]
    fn test_mask_short_strings() {
        assert_eq!("".mask(), "***");
        assert_eq!("a".mask(), "***");
        assert_eq!("short".mask(), "***");
        assert_eq!("12345".mask(), "***");
        assert_eq!("123456789012".mask(), "***"); // 恰好12个字符
    }

    #[test]
    fn test_mask_long_strings() {
        assert_eq!("1234567890123".mask(), "1234***0123"); // 13个字符
        assert_eq!("verylongapikey123456".mask(), "very***3456");
        assert_eq!("ghp_1234567890abcdefghijklmnop".mask(), "ghp_***mnop");
        assert_eq!(
            "sk-1234567890abcdefghijklmnopqrstuvwxyz".mask(),
            "sk-1***wxyz"
        );
    }

    #[test]
    fn test_mask_with_string_type() {
        let s = String::from("verylongapikey123456");
        assert_eq!(s.mask(), "very***3456");

        let short_string = String::from("short");
        assert_eq!(short_string.mask(), "***");
    }

    #[test]
    fn test_mask_backward_compatibility() {
        // 测试原有函数仍然可用
        assert_eq!(mask_sensitive_value("short"), "***");
        assert_eq!(mask_sensitive_value("verylongapikey123456"), "very***3456");
        assert_eq!(mask_sensitive_value(""), "***");
    }

    #[rstest]
    #[case("", "***")]
    #[case("a", "***")]
    #[case("abc", "***")]
    #[case("123456789012", "***")] // 12 chars
    #[case("1234567890123", "1234***0123")] // 13 chars
    #[case("abcdefghijklmnop", "abcd***mnop")] // 16 chars
    #[case("github_pat_1234567890abcdefghijklmnop", "gith***mnop")]
    #[case("very_long_api_key_with_underscores_123456", "very***3456")]
    fn test_mask_parametrized(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(input.mask(), expected);
        assert_eq!(mask_sensitive_value(input), expected);
    }

    #[test]
    fn test_mask_special_characters() {
        assert_eq!("key-with-dashes-123456789".mask(), "key-***6789");
        assert_eq!("key_with_underscores_123456".mask(), "key_***3456");
        assert_eq!("key.with.dots.123456789".mask(), "key.***6789");
        assert_eq!("key@with@symbols#123456".mask(), "key@***3456");
    }

    #[test]
    fn test_mask_unicode_strings() {
        assert_eq!("短字符串".mask(), "***");
        assert_eq!(
            "这是一个很长的中文字符串包含数字123456".mask(),
            "这是一个***3456"
        );
        assert_eq!("émoji🚀test123456789".mask(), "émoj***6789");
    }
}

#[cfg(test)]
mod date_format_tests {
    use super::*;

    // ==================== 日期时间格式化测试 ====================

    #[test]
    fn test_date_format_patterns() {
        // 测试日期格式
        let date_local = format_document_timestamp(DateFormat::DateOnly, Timezone::Local);
        let date_utc = format_document_timestamp(DateFormat::DateOnly, Timezone::Utc);

        // 验证格式：YYYY-MM-DD
        let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
        assert!(date_regex.is_match(&date_local));
        assert!(date_regex.is_match(&date_utc));
    }

    #[test]
    fn test_datetime_format_patterns() {
        // 测试日期时间格式
        let datetime_local = format_document_timestamp(DateFormat::DateTime, Timezone::Local);
        let datetime_utc = format_document_timestamp(DateFormat::DateTime, Timezone::Utc);

        // 验证格式：YYYY-MM-DD HH:MM:SS
        let datetime_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(datetime_regex.is_match(&datetime_local));
        assert!(datetime_regex.is_match(&datetime_utc));
    }

    #[test]
    fn test_iso8601_format_patterns() {
        // 测试 ISO 8601 格式
        let iso_local = format_document_timestamp(DateFormat::Iso8601, Timezone::Local);
        let iso_utc = format_document_timestamp(DateFormat::Iso8601, Timezone::Utc);

        // 验证 ISO 8601 格式特征
        assert!(iso_local.contains('T'));
        assert!(iso_utc.contains('T'));
        assert!(iso_utc.ends_with('Z') || iso_utc.contains('+') || iso_utc.contains('-'));
    }

    #[test]
    fn test_convenience_functions() {
        // 测试便利函数
        let last_updated = format_last_updated();
        let last_updated_with_time = format_last_updated_with_time();

        // 验证格式
        let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
        let datetime_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();

        assert!(date_regex.is_match(&last_updated));
        assert!(datetime_regex.is_match(&last_updated_with_time));
    }

    #[test]
    fn test_filename_timestamp_format() {
        let filename_timestamp = format_filename_timestamp();

        // 验证格式：YYYY-MM-DD_HH-MM-SS（适合文件名）
        let filename_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$").unwrap();
        assert!(filename_regex.is_match(&filename_timestamp));

        // 验证不包含空格和冒号（文件名友好）
        assert!(!filename_timestamp.contains(' '));
        assert!(!filename_timestamp.contains(':'));
    }

    #[test]
    fn test_date_consistency() {
        // 测试同一时刻的不同格式应该包含相同的日期部分
        let date_only = format_document_timestamp(DateFormat::DateOnly, Timezone::Local);
        let datetime = format_document_timestamp(DateFormat::DateTime, Timezone::Local);
        let filename_ts = format_filename_timestamp();

        // 提取日期部分进行比较
        let date_part_from_datetime = &datetime[..10];
        let date_part_from_filename = &filename_ts[..10];

        assert_eq!(date_only, date_part_from_datetime);
        assert_eq!(date_only, date_part_from_filename);
    }

    #[rstest]
    #[case(DateFormat::DateOnly, r"^\d{4}-\d{2}-\d{2}$")]
    #[case(DateFormat::DateTime, r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")]
    fn test_format_patterns_parametrized(#[case] format: DateFormat, #[case] pattern: &str) {
        let result_local = format_document_timestamp(format, Timezone::Local);
        let result_utc = format_document_timestamp(format, Timezone::Utc);

        let regex = regex::Regex::new(pattern).unwrap();
        assert!(regex.is_match(&result_local));
        assert!(regex.is_match(&result_utc));
    }
}

#[cfg(test)]
mod checksum_tests {
    use super::*;

    // ==================== 校验和计算测试 ====================

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
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = Checksum::verify(&file_path, wrong_hash);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("File integrity verification failed"));
        assert!(error_msg.contains("Expected:"));
        assert!(error_msg.contains("Actual:"));

        Ok(())
    }

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

    #[test]
    fn test_file_not_found() {
        let non_existent_path = Path::new("/this/path/does/not/exist/file.txt");
        let result = Checksum::calculate_file_sha256(non_existent_path);

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
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // ==================== 集成测试 ====================

    #[test]
    fn test_format_utilities_integration() {
        // 测试各种格式化工具的集成使用
        let file_size = DisplayFormatter::size(1024 * 1024 * 5); // 5MB
        let timestamp = format_filename_timestamp();
        let masked_key = "very_long_api_key_123456789".mask();

        // 验证格式化结果
        assert_eq!(file_size, "5.00 MB");
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}$")
            .unwrap()
            .is_match(&timestamp));
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
    }

    #[test]
    fn test_checksum_and_format_integration() -> Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("integration_test.txt");

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

    #[test]
    fn test_performance_characteristics() -> Result<()> {
        use std::time::Instant;

        // 测试格式化函数的性能特征（应该很快）
        let start = Instant::now();
        for i in 0..1000 {
            let _ = DisplayFormatter::size(i * 1024);
            let _ = format!("key_{}", i).mask();
        }
        let duration = start.elapsed();

        // 1000次格式化操作应该在很短时间内完成
        assert!(duration.as_millis() < 100);

        Ok(())
    }
}
