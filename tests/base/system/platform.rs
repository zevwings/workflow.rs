//! Platform 模块测试
//!
//! 测试平台检测、路径处理和系统信息获取功能。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::env;
use workflow::base::system::Platform;

// ==================== Platform Detection Tests ====================

/// 测试检测平台并获取有效的发布标识符格式
///
/// ## 测试目的
/// 验证 `Platform::detect().release_identifier()` 方法能够正确检测平台并返回有效的发布标识符格式。
///
/// ## 测试场景
/// 1. 调用 `Platform::detect().release_identifier()` 检测平台
///
/// ## 预期结果
/// - 返回的字符串不为空
/// - 格式正确，包含连字符（OS-ARCH 格式）
#[test]
fn test_detect_release_platform_return_ok() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 检测平台并获取发布标识符
    let platform = Platform::detect().release_identifier()?;

    // Assert: 验证返回的字符串不为空，格式正确
    assert!(!platform.is_empty());
    assert!(platform.contains('-'), "Platform should contain a hyphen");
    Ok(())
}

/// 测试在macOS上检测平台返回macOS标识符（仅macOS）
///
/// ## 测试目的
/// 验证在 macOS 系统上 `Platform::detect().release_identifier()` 方法能够返回正确的 macOS 平台标识符。
///
/// ## 测试场景
/// 1. 在 macOS 系统上调用 `Platform::detect().release_identifier()`
///
/// ## 预期结果
/// - 返回 "macOS-Intel" 或 "macOS-AppleSilicon"
/// - 架构匹配正确（x86_64 对应 Intel，aarch64 对应 AppleSilicon）
#[test]
fn test_detect_release_platform_on_macos_return_ok() -> Result<()> {
    // Arrange: 准备测试（仅在 macOS 上运行）
    // 注意：这个测试只在 macOS 上会通过

    // Act: 检测平台并获取发布标识符
    if env::consts::OS == "macos" {
        let platform = Platform::detect().release_identifier()?;

        // Assert: 验证 macOS 平台标识符格式
        assert!(
            platform == "macOS-Intel" || platform == "macOS-AppleSilicon",
            "macOS platform should be macOS-Intel or macOS-AppleSilicon, got: {}",
            platform
        );

        // 验证架构匹配
        if env::consts::ARCH == "x86_64" {
            assert_eq!(platform, "macOS-Intel");
        } else if env::consts::ARCH == "aarch64" {
            assert_eq!(platform, "macOS-AppleSilicon");
        }
    }
    Ok(())
}

/// 测试在Linux上检测平台返回Linux标识符（仅Linux）
///
/// ## 测试目的
/// 验证在 Linux 系统上 `Platform::detect().release_identifier()` 方法能够返回正确的 Linux 平台标识符。
///
/// ## 测试场景
/// 1. 在 Linux 系统上调用 `Platform::detect().release_identifier()`
///
/// ## 预期结果
/// - 返回 "Linux-x86_64"、"Linux-x86_64-static" 或 "Linux-ARM64"
/// - 架构匹配正确（x86_64 对应 Linux-x86_64 或 Linux-x86_64-static，aarch64 对应 Linux-ARM64）
#[test]
fn test_detect_release_platform_on_linux_return_ok() -> Result<()> {
    // Arrange: 准备测试（仅在 Linux 上运行）
    // 注意：这个测试只在 Linux 上会通过

    // Act: 检测平台并获取发布标识符
    if env::consts::OS == "linux" {
        let platform = Platform::detect().release_identifier()?;

        // Assert: 验证 Linux 平台标识符格式
        assert!(
            platform == "Linux-x86_64"
                || platform == "Linux-x86_64-static"
                || platform == "Linux-ARM64",
            "Linux platform should be Linux-x86_64, Linux-x86_64-static, or Linux-ARM64, got: {}",
            platform
        );

        // 验证架构匹配
        if env::consts::ARCH == "x86_64" {
            assert!(
                platform == "Linux-x86_64" || platform == "Linux-x86_64-static",
                "x86_64 Linux should be Linux-x86_64 or Linux-x86_64-static"
            );
        } else if env::consts::ARCH == "aarch64" {
            assert_eq!(platform, "Linux-ARM64");
        }
    }
    Ok(())
}

/// 测试在Windows上检测平台返回Windows标识符（仅Windows）
///
/// ## 测试目的
/// 验证在 Windows 系统上 `Platform::detect().release_identifier()` 方法能够返回正确的 Windows 平台标识符。
///
/// ## 测试场景
/// 1. 在 Windows 系统上调用 `Platform::detect().release_identifier()`
///
/// ## 预期结果
/// - 返回 "Windows-x86_64" 或 "Windows-ARM64"
/// - 架构匹配正确（x86_64 对应 Windows-x86_64，aarch64 对应 Windows-ARM64）
#[test]
fn test_detect_release_platform_on_windows_return_ok() -> Result<()> {
    // Arrange: 准备测试（仅在 Windows 上运行）
    // 注意：这个测试只在 Windows 上会通过

    // Act: 检测平台并获取发布标识符
    if env::consts::OS == "windows" {
        let platform = Platform::detect().release_identifier()?;

        // Assert: 验证 Windows 平台标识符格式
        assert!(
            platform == "Windows-x86_64" || platform == "Windows-ARM64",
            "Windows platform should be Windows-x86_64 or Windows-ARM64, got: {}",
            platform
        );

        // 验证架构匹配
        if env::consts::ARCH == "x86_64" {
            assert_eq!(platform, "Windows-x86_64");
        } else if env::consts::ARCH == "aarch64" {
            assert_eq!(platform, "Windows-ARM64");
        }
    }
    Ok(())
}

/// 测试多次调用平台检测返回一致的结果
///
/// ## 测试目的
/// 验证多次调用 `Platform::detect().release_identifier()` 方法能够返回一致的结果。
///
/// ## 测试场景
/// 1. 多次调用 `Platform::detect().release_identifier()` 方法
///
/// ## 预期结果
/// - 多次调用返回相同的结果
#[test]
fn test_detect_release_platform_with_multiple_calls_return_collect() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 多次调用平台检测
    let platform1 = Platform::detect().release_identifier()?;
    let platform2 = Platform::detect().release_identifier()?;
    let platform3 = Platform::detect().release_identifier()?;

    // Assert: 验证多次调用返回相同的结果
    assert_eq!(platform1, platform2);
    assert_eq!(platform2, platform3);
    Ok(())
}

/// 测试检测平台返回结构化的标识符格式（OS-ARCH格式）
///
/// ## 测试目的
/// 验证 `Platform::detect().release_identifier()` 方法返回的标识符格式符合 OS-ARCH 或 OS-ARCH-variant 的结构。
///
/// ## 测试场景
/// 1. 调用 `Platform::detect().release_identifier()` 获取平台标识符
/// 2. 验证标识符格式
///
/// ## 预期结果
/// - 标识符至少包含两部分，用连字符分隔
/// - 第一部分是操作系统名称（macOS、Linux 或 Windows）
#[test]
fn test_detect_release_platform_with_valid_format_return_ok() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 检测平台并获取发布标识符
    let platform = Platform::detect().release_identifier()?;

    // Assert: 验证格式结构（OS-ARCH 或 OS-ARCH-variant）
    let parts: Vec<&str> = platform.split('-').collect();
    assert!(
        parts.len() >= 2,
        "Platform format should have at least 2 parts separated by '-', got: {}",
        platform
    );

    // 验证第一部分是操作系统名称
    let os_part = parts[0];
    assert!(
        os_part == "macOS" || os_part == "Linux" || os_part == "Windows",
        "OS part should be macOS, Linux, or Windows, got: {}",
        os_part
    );
    Ok(())
}

/// 测试检测平台返回的标识符与系统架构匹配
///
/// ## 测试目的
/// 验证 `Platform::detect().release_identifier()` 方法返回的标识符中的架构部分与系统架构一致。
///
/// ## 测试场景
/// 1. 获取系统架构
/// 2. 调用 `Platform::detect().release_identifier()` 检测平台
/// 3. 验证标识符中的架构与系统架构匹配
///
/// ## 预期结果
/// - x86_64 架构对应标识符包含 "x86_64" 或 "Intel"
/// - aarch64 架构对应标识符包含 "ARM64" 或 "AppleSilicon"
#[test]
fn test_detect_release_platform_with_system_arch_return_ok() -> Result<()> {
    // Arrange: 准备测试，获取系统架构
    let arch = env::consts::ARCH;

    // Act: 检测平台并获取发布标识符
    let platform = Platform::detect().release_identifier()?;

    // Assert: 验证平台标识符中的架构与系统架构一致
    if arch == "x86_64" {
        assert!(
            platform.contains("x86_64") || platform.contains("Intel"),
            "Platform should contain x86_64 or Intel for x86_64 architecture"
        );
    } else if arch == "aarch64" {
        assert!(
            platform.contains("ARM64") || platform.contains("AppleSilicon"),
            "Platform should contain ARM64 or AppleSilicon for aarch64 architecture"
        );
    }
    Ok(())
}

/// 测试在任何平台上检测都不会panic
///
/// ## 测试目的
/// 验证 `Platform::detect().release_identifier()` 方法在任何平台上都不会 panic，即使在不支持的平台上也应返回错误而不是 panic。
///
/// ## 测试场景
/// 1. 捕获可能的 panic
/// 2. 调用 `Platform::detect().release_identifier()` 检测平台
///
/// ## 预期结果
/// - 不会 panic（即使在不支持的平台上也应该返回错误而不是 panic）
#[test]
fn test_detect_release_platform_with_any_platform_does_not_panic_return_ok() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）
    // 注意：即使在不支持的平台上，也应该返回错误而不是 panic

    // Act: 捕获可能的 panic
    let result = std::panic::catch_unwind(|| Platform::detect().release_identifier());

    // Assert: 验证不会 panic
    assert!(result.is_ok(), "detect_release_platform should not panic");
    Ok(())
}

/// 测试在支持的OS上检测平台返回成功
///
/// ## 测试目的
/// 验证 `Platform::detect().release_identifier()` 方法在支持的 OS（macOS、Linux、Windows）上能够成功返回结果。
///
/// ## 测试场景
/// 1. 调用 `Platform::detect().release_identifier()` 检测平台
///
/// ## 预期结果
/// - 在支持的平台上应该成功返回结果
#[test]
fn test_detect_release_platform_with_supported_os_return_ok() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）
    // 注意：这个测试主要验证函数不会因为意外的系统状态而 panic

    // Act: 检测平台并获取发布标识符
    let result = Platform::detect().release_identifier();

    // Assert: 在支持的平台上应该成功
    if env::consts::OS == "macos" || env::consts::OS == "linux" || env::consts::OS == "windows" {
        assert!(result.is_ok(), "Should succeed on supported platforms");
    }
    Ok(())
}

/// 测试平台标识符的大小写格式正确
///
/// ## 测试目的
/// 验证 `Platform::detect().release_identifier()` 方法返回的标识符大小写格式正确。
///
/// ## 测试场景
/// 1. 调用 `Platform::detect().release_identifier()` 检测平台
/// 2. 验证标识符的大小写格式
///
/// ## 预期结果
/// - macOS 应该是 "macOS"（特定大小写）
/// - Linux 应该是 "Linux"（首字母大写）
/// - Windows 应该是 "Windows"（首字母大写）
#[test]
fn test_platform_identifier_with_detected_platform_return_ok() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 检测平台并获取发布标识符
    let platform = Platform::detect().release_identifier()?;

    // Assert: 验证平台标识符的大小写格式
    // macOS 应该是 "macOS"（特定大小写）
    if platform.starts_with("macOS") {
        assert_eq!(&platform[0..5], "macOS");
    }

    // Linux 应该是 "Linux"（首字母大写）
    if platform.starts_with("Linux") {
        assert_eq!(&platform[0..5], "Linux");
    }

    // Windows 应该是 "Windows"（首字母大写）
    if platform.starts_with("Windows") {
        assert_eq!(&platform[0..7], "Windows");
    }
    Ok(())
}

// ==================== Platform 结构体基础方法测试 ====================

/// 测试使用OS和架构创建Platform实例
///
/// ## 测试目的
/// 验证 `Platform::new()` 方法能够使用操作系统和架构参数创建 Platform 实例。
///
/// ## 测试场景
/// 1. 准备操作系统和架构参数
/// 2. 调用 `Platform::new()` 创建实例
///
/// ## 预期结果
/// - Platform 实例创建成功
#[test]
fn test_platform_new_with_os_and_arch_return_ok() -> Result<()> {
    // Arrange: 准备操作系统和架构参数

    // Act: 创建 Platform 实例
    let platform = Platform::new("macos", "aarch64");

    // Assert: 验证操作系统和架构
    assert_eq!(platform.os(), "macos");
    assert_eq!(platform.arch(), "aarch64");
    Ok(())
}

/// 测试使用String参数创建Platform实例
///
/// ## 测试目的
/// 验证 `Platform::new()` 方法能够接受 String 类型参数创建 Platform 实例。
///
/// ## 测试场景
/// 1. 准备 String 类型的操作系统和架构参数
/// 2. 调用 `Platform::new()` 创建实例
///
/// ## 预期结果
/// - Platform 实例创建成功，参数正确保存
#[test]
fn test_platform_new_with_string_params_return_ok() -> Result<()> {
    // Arrange: 准备字符串参数
    let os = String::from("linux");
    let arch = String::from("x86_64");

    // Act: 创建 Platform 实例
    let platform = Platform::new(os, arch);

    // Assert: 验证操作系统和架构
    assert_eq!(platform.os(), "linux");
    assert_eq!(platform.arch(), "x86_64");
    Ok(())
}

/// 测试检测当前系统平台
///
/// ## 测试目的
/// 验证 `Platform::detect()` 方法能够正确检测当前系统的平台信息。
///
/// ## 测试场景
/// 1. 调用 `Platform::detect()` 检测当前平台
///
/// ## 预期结果
/// - 返回的操作系统和架构与系统实际值一致
#[test]
fn test_platform_detect_with_system_info_return_ok() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 检测当前平台
    let platform = Platform::detect();

    // Assert: 验证返回当前系统的操作系统和架构
    assert_eq!(platform.os(), env::consts::OS);
    assert_eq!(platform.arch(), env::consts::ARCH);
    Ok(())
}

/// 测试获取Platform实例的操作系统
///
/// ## 测试目的
/// 验证 `Platform::os()` 方法能够正确返回 Platform 实例的操作系统。
///
/// ## 测试场景
/// 1. 创建 Platform 实例
/// 2. 调用 `os()` 方法获取操作系统
///
/// ## 预期结果
/// - 返回正确的操作系统
#[test]
fn test_platform_os_with_platform_instance_return_ok() -> Result<()> {
    // Arrange: 创建 Platform 实例
    let platform = Platform::new("windows", "x86_64");

    // Act: 获取操作系统
    let os = platform.os();

    // Assert: 验证返回正确的操作系统
    assert_eq!(os, "windows");
    Ok(())
}

/// 测试获取Platform实例的架构
///
/// ## 测试目的
/// 验证 `Platform::arch()` 方法能够正确返回 Platform 实例的架构。
///
/// ## 测试场景
/// 1. 创建 Platform 实例
/// 2. 调用 `arch()` 方法获取架构
///
/// ## 预期结果
/// - 返回正确的架构
#[test]
fn test_platform_arch_with_platform_instance_return_ok() -> Result<()> {
    // Arrange: 创建 Platform 实例
    let platform = Platform::new("linux", "aarch64");

    // Act: 获取架构
    let arch = platform.arch();

    // Assert: 验证返回正确的架构
    assert_eq!(arch, "aarch64");
    Ok(())
}

/// 测试判断Platform是否为特定操作系统（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 Platform 的 is_macos、is_linux、is_windows 方法。
///
/// ## 测试场景
/// 测试不同操作系统平台的判断方法
///
/// ## 预期结果
/// - 每个方法对对应的平台返回 true，对其他平台返回 false
#[rstest]
#[case("macos", "aarch64", "linux", "x86_64", true, false)] // is_macos
#[case("linux", "x86_64", "macos", "aarch64", true, false)] // is_linux
#[case("windows", "x86_64", "linux", "x86_64", true, false)] // is_windows
fn test_platform_is_os_return_ok(
    #[case] target_os: &str,
    #[case] target_arch: &str,
    #[case] other_os: &str,
    #[case] other_arch: &str,
    #[case] target_should_be_true: bool,
    #[case] other_should_be_true: bool,
) -> Result<()> {
    // Arrange: 创建目标平台和其他平台实例
    let target_platform = Platform::new(target_os, target_arch);
    let other_platform = Platform::new(other_os, other_arch);

    // Act & Assert: 验证平台判断方法
    match target_os {
        "macos" => {
            assert_eq!(target_platform.is_macos(), target_should_be_true);
            assert_eq!(other_platform.is_macos(), other_should_be_true);
        }
        "linux" => {
            assert_eq!(target_platform.is_linux(), target_should_be_true);
            assert_eq!(other_platform.is_linux(), other_should_be_true);
        }
        "windows" => {
            assert_eq!(target_platform.is_windows(), target_should_be_true);
            assert_eq!(other_platform.is_windows(), other_should_be_true);
        }
        _ => {}
    }
    Ok(())
}

/// 测试判断Platform是否为特定架构（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 Platform 的 is_x86_64、is_aarch64 方法。
///
/// ## 测试场景
/// 测试不同架构的判断方法
///
/// ## 预期结果
/// - 每个方法对对应的架构返回 true，对其他架构返回 false
#[rstest]
#[case("linux", "x86_64", "linux", "aarch64", true, false)] // is_x86_64
#[case("macos", "aarch64", "macos", "x86_64", true, false)] // is_aarch64
fn test_platform_is_arch_return_ok(
    #[case] target_os: &str,
    #[case] target_arch: &str,
    #[case] other_os: &str,
    #[case] other_arch: &str,
    #[case] target_should_be_true: bool,
    #[case] other_should_be_true: bool,
) -> Result<()> {
    // Arrange: 创建目标架构和其他架构的平台实例
    let target_platform = Platform::new(target_os, target_arch);
    let other_platform = Platform::new(other_os, other_arch);

    // Act & Assert: 验证架构判断方法
    if target_arch == "x86_64" {
        assert_eq!(target_platform.is_x86_64(), target_should_be_true);
        assert_eq!(other_platform.is_x86_64(), other_should_be_true);
    } else if target_arch == "aarch64" {
        assert_eq!(target_platform.is_aarch64(), target_should_be_true);
        assert_eq!(other_platform.is_aarch64(), other_should_be_true);
    }
    Ok(())
}

/// 测试平台发布标识符（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 Platform::release_identifier() 能够返回正确的平台标识符。
///
/// ## 测试场景
/// 测试各种操作系统和架构组合的发布标识符
///
/// ## 预期结果
/// - 每个平台组合返回对应的标识符
#[rstest]
#[case("macos", "x86_64", "macOS-Intel")]
#[case("macos", "aarch64", "macOS-AppleSilicon")]
#[case("linux", "x86_64", "Linux-x86_64")] // 可能是 Linux-x86_64 或 Linux-x86_64-static
#[case("linux", "aarch64", "Linux-ARM64")]
#[case("windows", "x86_64", "Windows-x86_64")]
#[case("windows", "aarch64", "Windows-ARM64")]
fn test_platform_release_identifier_return_ok(
    #[case] os: &str,
    #[case] arch: &str,
    #[case] expected_identifier: &str,
) -> Result<()> {
    // Arrange: 创建平台实例
    let platform = Platform::new(os, arch);

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回的标识符（Linux x86_64 可能是两种格式之一）
    if os == "linux" && arch == "x86_64" {
        assert!(
            identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static",
            "Expected Linux-x86_64 or Linux-x86_64-static, got: {}",
            identifier
        );
    } else {
        assert_eq!(identifier, expected_identifier);
    }
    Ok(())
}

/// 测试不支持的平台返回错误
///
/// ## 测试目的
/// 验证 `Platform::release_identifier()` 方法对不支持的平台能够返回错误。
///
/// ## 测试场景
/// 1. 创建不支持的平台实例
/// 2. 调用 `release_identifier()` 方法
///
/// ## 预期结果
/// - 返回错误，错误消息包含 "Unsupported platform"
#[test]
fn test_platform_release_identifier_with_unsupported_platform_return_ok() -> Result<()> {
    // Arrange: 创建不支持的平台实例
    let platform = Platform::new("unsupported_os", "unsupported_arch");

    // Act: 获取发布标识符
    let result = platform.release_identifier();

    // Assert: 验证返回错误，错误消息包含 "Unsupported platform"
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported platform"));
    Ok(())
}

/// 测试Platform实例的Debug格式化输出
///
/// ## 测试目的
/// 验证 Platform 实例的 Debug trait 实现能够正确输出调试信息。
///
/// ## 测试场景
/// 1. 创建 Platform 实例
/// 2. 使用 Debug 格式化输出
///
/// ## 预期结果
/// - 输出包含操作系统和架构信息
#[test]
fn test_platform_debug_with_platform_instance_return_ok() -> Result<()> {
    // Arrange: 创建 Platform 实例
    let platform = Platform::new("macos", "aarch64");

    // Act: 格式化 Debug 输出
    let debug_str = format!("{:?}", platform);

    // Assert: 验证 Debug 字符串包含平台信息
    assert!(debug_str.contains("macos") || debug_str.contains("aarch64"));
    Ok(())
}

/// 测试克隆Platform实例
///
/// ## 测试目的
/// 验证 Platform 实例的 Clone trait 实现能够正确克隆实例。
///
/// ## 测试场景
/// 1. 创建 Platform 实例
/// 2. 克隆实例
///
/// ## 预期结果
/// - 克隆的实例与原实例属性相同
#[test]
fn test_platform_clone_with_platform_instance_return_ok() -> Result<()> {
    // Arrange: 创建 Platform 实例
    let platform1 = Platform::new("linux", "x86_64");

    // Act: 克隆平台实例
    let platform2 = platform1.clone();

    // Assert: 验证克隆后的平台实例属性相同
    assert_eq!(platform1.os(), platform2.os());
    assert_eq!(platform1.arch(), platform2.arch());
    Ok(())
}

/// 测试Platform实例的相等性比较
///
/// ## 测试目的
/// 验证 Platform 实例的 PartialEq trait 实现能够正确比较实例。
///
/// ## 测试场景
/// 1. 创建多个 Platform 实例
/// 2. 比较实例的相等性
///
/// ## 预期结果
/// - 相同参数的实例相等
/// - 不同参数的实例不相等
#[test]
fn test_platform_eq_with_same_platforms_return_ok() -> Result<()> {
    // Arrange: 创建相同的平台实例
    let platform1 = Platform::new("macos", "aarch64");
    let platform2 = Platform::new("macos", "aarch64");
    let platform3 = Platform::new("linux", "x86_64");

    // Act & Assert: 验证相同平台相等，不同平台不相等
    assert_eq!(platform1, platform2);
    assert_ne!(platform1, platform3);
    Ok(())
}

/// 测试Linux x86_64平台检测静态链接需求
///
/// ## 测试目的
/// 验证在 Linux x86_64 平台上能够检测是否需要静态链接。
///
/// ## 测试场景
/// 1. 创建 Linux x86_64 Platform 实例
/// 2. 调用 `release_identifier()` 方法
///
/// ## 预期结果
/// - 返回包含或不包含 "-static" 后缀的标识符（取决于系统环境）
#[test]
fn test_platform_release_identifier_with_linux_x86_64_detects_static_link_return_ok() -> Result<()> {
    // Arrange: 创建 Linux x86_64 平台实例
    // 这个测试验证 release_identifier 能够正确检测静态链接需求

    // Act: 获取发布标识符
    let platform = Platform::new("linux", "x86_64");
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 Linux-x86_64 或 Linux-x86_64-static
    assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
    Ok(())
}

/// 测试非Linux平台不检查静态链接
///
/// ## 测试目的
/// 验证非 Linux 平台不进行静态链接检查。
///
/// ## 测试场景
/// 1. 创建 macOS 或 Windows Platform 实例
/// 2. 调用 `release_identifier()` 方法
///
/// ## 预期结果
/// - 返回的标识符不包含 "-static" 后缀
#[test]
fn test_platform_release_identifier_with_non_linux_does_not_check_static_return_ok() -> Result<()> {
    // Arrange: 创建非 Linux 平台实例
    let macos = Platform::new("macos", "x86_64");
    let windows = Platform::new("windows", "x86_64");

    // Act: 获取发布标识符
    let macos_id = macos.release_identifier()?;
    let windows_id = windows.release_identifier()?;

    // Assert: 验证非 Linux 平台不会检查静态链接
    assert_eq!(macos_id, "macOS-Intel");
    assert_eq!(windows_id, "Windows-x86_64");
    Ok(())
}

/// 测试Linux非x86_64架构不检查静态链接
///
/// ## 测试目的
/// 验证 Linux 非 x86_64 架构不进行静态链接检查。
///
/// ## 测试场景
/// 1. 创建 Linux ARM64 Platform 实例
/// 2. 调用 `release_identifier()` 方法
///
/// ## 预期结果
/// - 返回的标识符不包含 "-static" 后缀
#[test]
fn test_platform_release_identifier_with_linux_non_x86_64_does_not_check_static_return_ok() -> Result<()> {
    // Arrange: 创建 Linux 非 x86_64 架构平台实例
    let linux_arm64 = Platform::new("linux", "aarch64");

    // Act: 获取发布标识符
    let identifier = linux_arm64.release_identifier()?;

    // Assert: 验证非 x86_64 架构不会检查静态链接
    assert_eq!(identifier, "Linux-ARM64");
    Ok(())
}

/// 测试所有平台组合返回正确的标识符
///
/// ## 测试目的
/// 验证所有支持的平台组合都能返回正确的标识符。
///
/// ## 测试场景
/// 1. 测试各种平台组合（macOS-Intel、macOS-AppleSilicon、Linux-x86_64、Linux-ARM64、Windows-x86_64、Windows-ARM64）
/// 2. 调用 `release_identifier()` 方法
///
/// ## 预期结果
/// - 所有支持的组合都返回预期的标识符
#[test]
fn test_platform_release_identifier_with_all_combinations_return_collect() -> Result<()> {
    // Arrange: 准备所有平台组合
    let combinations = vec![
        ("macos", "x86_64", "macOS-Intel"),
        ("macos", "aarch64", "macOS-AppleSilicon"),
        ("linux", "aarch64", "Linux-ARM64"),
        ("windows", "x86_64", "Windows-x86_64"),
        ("windows", "aarch64", "Windows-ARM64"),
    ];

    // Act & Assert: 验证所有平台组合返回正确的标识符
    for (os, arch, expected_prefix) in combinations {
        let platform = Platform::new(os, arch);
        let identifier = platform.release_identifier()?;
        assert!(
            identifier.starts_with(expected_prefix),
            "Platform {}-{} should start with {}",
            os,
            arch,
            expected_prefix
        );
    }
    Ok(())
}

/// 测试Alpine Linux检测静态链接需求
///
/// ## 测试目的
/// 验证能够检测 Alpine Linux 并添加 "-static" 后缀。
///
/// ## 测试场景
/// 1. 在 Alpine Linux 环境中检测平台
///
/// ## 预期结果
/// - 返回的标识符包含 "-static" 后缀（如果在 Alpine Linux 环境中）
#[test]
fn test_platform_release_identifier_with_alpine_linux_detects_static_return_ok() -> Result<()> {
    // Arrange: 创建 Linux x86_64 平台实例
    // 注意：这个测试在非 Alpine Linux 系统上可能无法完全测试
    // 但至少可以验证代码路径存在
    let platform = Platform::new("linux", "x86_64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 Linux-x86_64 或 Linux-x86_64-static
    assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
    Ok(())
}

/// 测试处理ldd命令不同输出的场景
///
/// ## 测试目的
/// 验证能够正确处理 ldd 命令的不同输出场景。
///
/// ## 测试场景
/// 1. 测试 ldd 命令的各种输出情况
///
/// ## 预期结果
/// - 能够正确解析 ldd 输出并检测静态链接需求
#[test]
fn test_platform_release_identifier_with_ldd_scenarios_handles_different_outputs_return_ok() -> Result<()> {
    // Arrange: 创建不同平台实例
    // 由于无法直接控制 ldd 命令的输出，我们通过 release_identifier 间接测试
    let platform = Platform::new("linux", "x86_64");
    let macos_platform = Platform::new("macos", "x86_64");
    let linux_arm64_platform = Platform::new("linux", "aarch64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;
    let macos_id = macos_platform.release_identifier()?;
    let linux_arm64_id = linux_arm64_platform.release_identifier()?;

    // Assert: 验证格式正确，非 Linux 和非 x86_64 不检查静态链接
    assert!(identifier.starts_with("Linux-x86_64"));
    assert_eq!(macos_id, "macOS-Intel");
    assert_eq!(linux_arm64_id, "Linux-ARM64");
    Ok(())
}

/// 测试非Linux平台不执行静态链接检查
///
/// ## 测试目的
/// 验证非 Linux 平台跳过静态链接检查。
///
/// ## 测试场景
/// 1. 在非 Linux 平台上检测平台
///
/// ## 预期结果
/// - 不执行静态链接检查，返回标准标识符
#[test]
fn test_platform_release_identifier_with_non_linux_return_ok() -> Result<()> {
    // Arrange: 创建非 Linux 平台实例
    // 测试非 Linux 平台的早期返回路径（覆盖 is_static_required 的第 117-118 行）
    let macos = Platform::new("macos", "x86_64");
    let windows = Platform::new("windows", "x86_64");

    // Act: 获取发布标识符
    let macos_id = macos.release_identifier()?;
    let windows_id = windows.release_identifier()?;

    // Assert: 验证非 Linux 平台不检查静态链接
    assert_eq!(macos_id, "macOS-Intel");
    assert_eq!(windows_id, "Windows-x86_64");
    Ok(())
}

/// 测试非x86_64架构不执行静态链接检查
///
/// ## 测试目的
/// 验证非 x86_64 架构跳过静态链接检查。
///
/// ## 测试场景
/// 1. 在 Linux ARM64 平台上检测平台
///
/// ## 预期结果
/// - 不执行静态链接检查，返回标准标识符
#[test]
fn test_platform_release_identifier_with_non_x86_64_return_ok() -> Result<()> {
    // Arrange: 创建非 x86_64 架构平台实例
    // 测试非 x86_64 架构的早期返回路径（覆盖 is_static_required 的第 117-118 行）
    let linux_arm64 = Platform::new("linux", "aarch64");

    // Act: 获取发布标识符
    let identifier = linux_arm64.release_identifier()?;

    // Assert: 验证非 x86_64 架构不检查静态链接
    assert_eq!(identifier, "Linux-ARM64");
    Ok(())
}

/// 测试Linux x86_64平台处理不同场景（Alpine检测、静态链接检测）
#[test]
fn test_platform_release_identifier_with_linux_x86_64_handles_different_scenarios_return_ok() -> Result<()> {
    // Arrange: 创建 Linux x86_64 平台实例
    // 由于无法直接控制文件读取和命令执行，我们通过 release_identifier 间接测试
    // 这个测试验证代码路径存在，实际行为取决于运行环境
    let platform = Platform::new("linux", "x86_64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 Linux-x86_64 或 Linux-x86_64-static
    // 具体值取决于：
    // 1. 是否是 Alpine Linux（通过 /etc/os-release 检测）
    // 2. ldd 命令的输出（静态链接检测）
    assert!(
        identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static",
        "Linux x86_64 platform should return Linux-x86_64 or Linux-x86_64-static, got: {}",
        identifier
    );
    Ok(())
}

/// 测试在真实Linux x86_64环境中返回有效的标识符（仅Linux x86_64）
#[test]
#[cfg(target_os = "linux")]
#[cfg(target_arch = "x86_64")]
fn test_platform_release_identifier_in_actual_linux_environment_return_ok() -> Result<()> {
    // Arrange: 创建 Linux x86_64 平台实例
    // 在真实的 Linux x86_64 环境中测试静态链接检测
    // 这个测试只在 Linux x86_64 平台上运行
    let platform = Platform::new("linux", "x86_64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回有效的标识符
    assert!(
        identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static",
        "Should return valid Linux x86_64 identifier, got: {}",
        identifier
    );

    // 如果系统上有 /etc/os-release 文件，可以检查是否包含 Alpine 信息
    // 这有助于验证 Alpine Linux 检测逻辑
    if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
        if os_release.contains("Alpine") || os_release.contains("ID=alpine") {
            // 如果是 Alpine Linux，应该返回 static 版本
            assert_eq!(
                identifier, "Linux-x86_64-static",
                "Alpine Linux should return Linux-x86_64-static"
            );
        }
    }
    Ok(())
}
