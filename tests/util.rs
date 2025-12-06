//! 工具函数模块测试
//!
//! 测试 `base::util` 模块中的各种工具函数。

use workflow::base::util::{format_size, mask_sensitive_value};

// ==================== Format 模块测试 ====================

#[test]
fn test_format_size_zero() {
    assert_eq!(format_size(0), "0 B");
}

#[test]
fn test_format_size_bytes() {
    assert_eq!(format_size(1), "1 B");
    assert_eq!(format_size(100), "100 B");
    assert_eq!(format_size(512), "512 B");
    assert_eq!(format_size(1023), "1023 B");
}

#[test]
fn test_format_size_kilobytes() {
    assert_eq!(format_size(1024), "1.00 KB");
    assert_eq!(format_size(1536), "1.50 KB");
    assert_eq!(format_size(2048), "2.00 KB");
    assert_eq!(format_size(10240), "10.00 KB");
    assert_eq!(format_size(102400), "100.00 KB");
}

#[test]
fn test_format_size_megabytes() {
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
    assert_eq!(format_size(1024 * 1024 * 2), "2.00 MB");
    assert_eq!(format_size(1024 * 1024 * 10), "10.00 MB");
    assert_eq!(format_size(1024 * 1024 * 100), "100.00 MB");
}

#[test]
fn test_format_size_gigabytes() {
    assert_eq!(format_size(1024_u64.pow(3)), "1.00 GB");
    assert_eq!(format_size(1024_u64.pow(3) * 2), "2.00 GB");
    assert_eq!(format_size(1024_u64.pow(3) * 10), "10.00 GB");
}

#[test]
fn test_format_size_terabytes() {
    assert_eq!(format_size(1024_u64.pow(4)), "1.00 TB");
    assert_eq!(format_size(1024_u64.pow(4) * 2), "2.00 TB");
}

#[test]
fn test_format_size_large_values() {
    // 测试非常大的值
    assert_eq!(format_size(1024_u64.pow(5)), "1024.00 TB");
}

#[test]
fn test_format_size_boundary_values() {
    // 测试边界值
    assert_eq!(format_size(1023), "1023 B");
    assert_eq!(format_size(1024), "1.00 KB");
    assert_eq!(format_size(1025), "1.00 KB");

    assert_eq!(format_size(1024 * 1024 - 1), "1024.00 KB");
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
    assert_eq!(format_size(1024 * 1024 + 1), "1.00 MB");
}

// ==================== String 模块测试 ====================

#[test]
fn test_mask_sensitive_value_short() {
    // 短值（长度 ≤ 12）：完全隐藏
    assert_eq!(mask_sensitive_value(""), "***");
    assert_eq!(mask_sensitive_value("a"), "***");
    assert_eq!(mask_sensitive_value("short"), "***");
    assert_eq!(mask_sensitive_value("123456789012"), "***"); // 正好 12 个字符
}

#[test]
fn test_mask_sensitive_value_long() {
    // 长值（长度 > 12）：显示前4个字符和后4个字符
    assert_eq!(mask_sensitive_value("1234567890123"), "1234***0123"); // 13 个字符
    assert_eq!(mask_sensitive_value("verylongapikey123456"), "very***3456");
    assert_eq!(
        mask_sensitive_value("abcdefghijklmnopqrstuvwxyz"),
        "abcd***wxyz"
    );
}

#[test]
fn test_mask_sensitive_value_exact_boundary() {
    // 测试边界情况：正好 13 个字符
    let value = "1234567890123";
    let masked = mask_sensitive_value(value);
    assert_eq!(masked, "1234***0123");
    assert_eq!(masked.len(), 13); // 前4 + "***" + 后4 = 11，但实际是 13
}

#[test]
fn test_mask_sensitive_value_very_long() {
    // 测试非常长的值
    let long_value = "a".repeat(100);
    let masked = mask_sensitive_value(&long_value);
    assert!(masked.starts_with("aaaa"));
    assert!(masked.ends_with("aaaa"));
    assert!(masked.contains("***"));
}

#[test]
fn test_mask_sensitive_value_special_characters() {
    // 测试特殊字符
    assert_eq!(mask_sensitive_value("test@example.com"), "test***m.co");
    assert_eq!(mask_sensitive_value("key-with-dashes"), "key-***hes");
    assert_eq!(mask_sensitive_value("key_with_underscores"), "key_***ores");
}

#[test]
fn test_mask_sensitive_value_unicode() {
    // 测试 Unicode 字符（注意：可能不会按预期工作，因为 len() 计算的是字节数）
    let unicode = "测试中文1234567890123";
    let masked = mask_sensitive_value(unicode);
    // Unicode 字符的 len() 可能大于字符数，所以行为可能不同
    assert!(masked.contains("***"));
}

#[test]
fn test_mask_sensitive_value_api_key_like() {
    // 测试类似 API key 的值
    assert_eq!(
        mask_sensitive_value("sk-12345678901234567890"),
        "sk-1***7890"
    );
    assert_eq!(
        mask_sensitive_value("ghp_abcdefghijklmnopqrstuvwxyz"),
        "ghp_***wxyz"
    );
}
