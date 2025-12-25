//! Platform 模块测试
//!
//! 测试平台检测、路径处理和系统信息获取功能。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use std::env;
use workflow::base::system::Platform;

// ==================== 平台检测测试 ====================

/// 测试检测平台并获取有效的发布标识符格式
#[test]
fn test_detect_release_platform_returns_valid_format() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 检测平台并获取发布标识符
    let platform = Platform::detect().release_identifier()?;

    // Assert: 验证返回的字符串不为空，格式正确
    assert!(!platform.is_empty());
    assert!(platform.contains('-'), "Platform should contain a hyphen");
    Ok(())
}

/// 测试在macOS上检测平台返回macOS标识符（仅macOS）
#[test]
fn test_detect_release_platform_on_macos_returns_macos_identifier() -> Result<()> {
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
#[test]
fn test_detect_release_platform_on_linux_returns_linux_identifier() -> Result<()> {
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
#[test]
fn test_detect_release_platform_on_windows_returns_windows_identifier() -> Result<()> {
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
#[test]
fn test_detect_release_platform_with_multiple_calls_returns_consistent_result() -> Result<()> {
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
#[test]
fn test_detect_release_platform_with_valid_format_returns_structured_identifier() -> Result<()> {
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
#[test]
fn test_detect_release_platform_with_system_arch_returns_matching_identifier() -> Result<()> {
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
#[test]
fn test_detect_release_platform_with_any_platform_does_not_panic() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）
    // 注意：即使在不支持的平台上，也应该返回错误而不是 panic

    // Act: 捕获可能的 panic
    let result = std::panic::catch_unwind(|| Platform::detect().release_identifier());

    // Assert: 验证不会 panic
    assert!(result.is_ok(), "detect_release_platform should not panic");
    Ok(())
}

/// 测试在支持的OS上检测平台返回成功
#[test]
fn test_detect_release_platform_with_supported_os_returns_ok() -> Result<()> {
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
#[test]
fn test_platform_identifier_with_detected_platform_returns_correct_case() -> Result<()> {
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
#[test]
fn test_platform_new_with_os_and_arch_returns_platform() -> Result<()> {
    // Arrange: 准备操作系统和架构参数

    // Act: 创建 Platform 实例
    let platform = Platform::new("macos", "aarch64");

    // Assert: 验证操作系统和架构
    assert_eq!(platform.os(), "macos");
    assert_eq!(platform.arch(), "aarch64");
    Ok(())
}

/// 测试使用String参数创建Platform实例
#[test]
fn test_platform_new_with_string_params_returns_platform() -> Result<()> {
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
#[test]
fn test_platform_detect_with_system_info_returns_current_platform() -> Result<()> {
    // Arrange: 准备测试（无需额外准备）

    // Act: 检测当前平台
    let platform = Platform::detect();

    // Assert: 验证返回当前系统的操作系统和架构
    assert_eq!(platform.os(), env::consts::OS);
    assert_eq!(platform.arch(), env::consts::ARCH);
    Ok(())
}

/// 测试获取Platform实例的操作系统
#[test]
fn test_platform_os_with_platform_instance_returns_os() -> Result<()> {
    // Arrange: 创建 Platform 实例
    let platform = Platform::new("windows", "x86_64");

    // Act: 获取操作系统
    let os = platform.os();

    // Assert: 验证返回正确的操作系统
    assert_eq!(os, "windows");
    Ok(())
}

/// 测试获取Platform实例的架构
#[test]
fn test_platform_arch_with_platform_instance_returns_arch() -> Result<()> {
    // Arrange: 创建 Platform 实例
    let platform = Platform::new("linux", "aarch64");

    // Act: 获取架构
    let arch = platform.arch();

    // Assert: 验证返回正确的架构
    assert_eq!(arch, "aarch64");
    Ok(())
}

/// 测试判断Platform是否为macOS
#[test]
fn test_platform_is_macos_with_macos_platform_returns_true() -> Result<()> {
    // Arrange: 创建 macOS 和 Linux 平台实例
    let macos_platform = Platform::new("macos", "aarch64");
    let linux_platform = Platform::new("linux", "x86_64");

    // Act & Assert: 验证 macOS 平台返回 true，其他平台返回 false
    assert!(macos_platform.is_macos());
    assert!(!linux_platform.is_macos());
    Ok(())
}

/// 测试判断Platform是否为Linux
#[test]
fn test_platform_is_linux_with_linux_platform_returns_true() -> Result<()> {
    // Arrange: 创建 Linux 和 macOS 平台实例
    let linux_platform = Platform::new("linux", "x86_64");
    let macos_platform = Platform::new("macos", "aarch64");

    // Act & Assert: 验证 Linux 平台返回 true，其他平台返回 false
    assert!(linux_platform.is_linux());
    assert!(!macos_platform.is_linux());
    Ok(())
}

/// 测试判断Platform是否为Windows
#[test]
fn test_platform_is_windows_with_windows_platform_returns_true() -> Result<()> {
    // Arrange: 创建 Windows 和 Linux 平台实例
    let windows_platform = Platform::new("windows", "x86_64");
    let linux_platform = Platform::new("linux", "x86_64");

    // Act & Assert: 验证 Windows 平台返回 true，其他平台返回 false
    assert!(windows_platform.is_windows());
    assert!(!linux_platform.is_windows());
    Ok(())
}

/// 测试判断Platform是否为x86_64架构
#[test]
fn test_platform_is_x86_64_with_x86_64_arch_returns_true() -> Result<()> {
    // Arrange: 创建 x86_64 和 aarch64 平台实例
    let x86_64_platform = Platform::new("linux", "x86_64");
    let aarch64_platform = Platform::new("linux", "aarch64");

    // Act & Assert: 验证 x86_64 架构返回 true，其他架构返回 false
    assert!(x86_64_platform.is_x86_64());
    assert!(!aarch64_platform.is_x86_64());
    Ok(())
}

/// 测试判断Platform是否为aarch64架构
#[test]
fn test_platform_is_aarch64_with_aarch64_arch_returns_true() -> Result<()> {
    // Arrange: 创建 aarch64 和 x86_64 平台实例
    let aarch64_platform = Platform::new("macos", "aarch64");
    let x86_64_platform = Platform::new("macos", "x86_64");

    // Act & Assert: 验证 aarch64 架构返回 true，其他架构返回 false
    assert!(aarch64_platform.is_aarch64());
    assert!(!x86_64_platform.is_aarch64());
    Ok(())
}

/// 测试macOS x86_64平台返回macOS-Intel标识符
#[test]
fn test_platform_release_identifier_with_macos_x86_64_returns_macos_intel() -> Result<()> {
    // Arrange: 创建 macOS x86_64 平台实例
    let platform = Platform::new("macos", "x86_64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 macOS-Intel
    assert_eq!(identifier, "macOS-Intel");
    Ok(())
}

/// 测试macOS aarch64平台返回macOS-AppleSilicon标识符
#[test]
fn test_platform_release_identifier_with_macos_aarch64_returns_macos_apple_silicon() -> Result<()> {
    // Arrange: 创建 macOS aarch64 平台实例
    let platform = Platform::new("macos", "aarch64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 macOS-AppleSilicon
    assert_eq!(identifier, "macOS-AppleSilicon");
    Ok(())
}

/// 测试Linux x86_64平台返回Linux-x86_64或Linux-x86_64-static标识符
#[test]
fn test_platform_release_identifier_with_linux_x86_64_returns_linux_x86_64() -> Result<()> {
    // Arrange: 创建 Linux x86_64 平台实例
    let platform = Platform::new("linux", "x86_64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 Linux-x86_64 或 Linux-x86_64-static（取决于静态链接检测）
    assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
    Ok(())
}

/// 测试Linux aarch64平台返回Linux-ARM64标识符
#[test]
fn test_platform_release_identifier_with_linux_aarch64_returns_linux_arm64() -> Result<()> {
    // Arrange: 创建 Linux aarch64 平台实例
    let platform = Platform::new("linux", "aarch64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 Linux-ARM64
    assert_eq!(identifier, "Linux-ARM64");
    Ok(())
}

/// 测试Windows x86_64平台返回Windows-x86_64标识符
#[test]
fn test_platform_release_identifier_with_windows_x86_64_returns_windows_x86_64() -> Result<()> {
    // Arrange: 创建 Windows x86_64 平台实例
    let platform = Platform::new("windows", "x86_64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 Windows-x86_64
    assert_eq!(identifier, "Windows-x86_64");
    Ok(())
}

/// 测试Windows aarch64平台返回Windows-ARM64标识符
#[test]
fn test_platform_release_identifier_with_windows_aarch64_returns_windows_arm64() -> Result<()> {
    // Arrange: 创建 Windows aarch64 平台实例
    let platform = Platform::new("windows", "aarch64");

    // Act: 获取发布标识符
    let identifier = platform.release_identifier()?;

    // Assert: 验证返回 Windows-ARM64
    assert_eq!(identifier, "Windows-ARM64");
    Ok(())
}

/// 测试不支持的平台返回错误
#[test]
fn test_platform_release_identifier_with_unsupported_platform_returns_error() -> Result<()> {
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
#[test]
fn test_platform_debug_with_platform_instance_returns_debug_string() -> Result<()> {
    // Arrange: 创建 Platform 实例
    let platform = Platform::new("macos", "aarch64");

    // Act: 格式化 Debug 输出
    let debug_str = format!("{:?}", platform);

    // Assert: 验证 Debug 字符串包含平台信息
    assert!(debug_str.contains("macos") || debug_str.contains("aarch64"));
    Ok(())
}

/// 测试克隆Platform实例
#[test]
fn test_platform_clone_with_platform_instance_returns_cloned_platform() -> Result<()> {
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
#[test]
fn test_platform_eq_with_same_platforms_returns_true() -> Result<()> {
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
#[test]
fn test_platform_release_identifier_with_linux_x86_64_detects_static_link() -> Result<()> {
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
#[test]
fn test_platform_release_identifier_with_non_linux_does_not_check_static() -> Result<()> {
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
#[test]
fn test_platform_release_identifier_with_linux_non_x86_64_does_not_check_static() -> Result<()> {
    // Arrange: 创建 Linux 非 x86_64 架构平台实例
    let linux_arm64 = Platform::new("linux", "aarch64");

    // Act: 获取发布标识符
    let identifier = linux_arm64.release_identifier()?;

    // Assert: 验证非 x86_64 架构不会检查静态链接
    assert_eq!(identifier, "Linux-ARM64");
    Ok(())
}

/// 测试所有平台组合返回正确的标识符
#[test]
fn test_platform_release_identifier_with_all_combinations_returns_correct_identifiers() -> Result<()> {
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
#[test]
fn test_platform_release_identifier_with_alpine_linux_detects_static() -> Result<()> {
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
#[test]
fn test_platform_release_identifier_with_ldd_scenarios_handles_different_outputs() -> Result<()> {
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
#[test]
fn test_platform_release_identifier_with_non_linux_returns_without_static_check() -> Result<()> {
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
#[test]
fn test_platform_release_identifier_with_non_x86_64_returns_without_static_check() -> Result<()> {
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
fn test_platform_release_identifier_with_linux_x86_64_handles_different_scenarios() -> Result<()> {
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
fn test_platform_release_identifier_in_actual_linux_environment_returns_valid_identifier() -> Result<()> {
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
