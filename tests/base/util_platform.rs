//! Platform 模块测试
//!
//! 测试平台检测、路径处理和系统信息获取功能。

use std::env;
use workflow::base::util::platform::detect_release_platform;

// ==================== 平台检测测试 ====================

#[test]
fn test_detect_release_platform_returns_valid_format() {
    // 测试平台检测返回有效的格式
    let platform = detect_release_platform().expect("Should detect platform");

    // 验证返回的字符串不为空
    assert!(!platform.is_empty());

    // 验证格式：应该包含平台名称和架构，用连字符分隔
    assert!(platform.contains('-'), "Platform should contain a hyphen");
}

#[test]
fn test_detect_release_platform_macos() {
    // 测试 macOS 平台检测
    // 注意：这个测试只在 macOS 上会通过
    if env::consts::OS == "macos" {
        let platform = detect_release_platform().expect("Should detect macOS platform");

        // macOS 应该是 macOS-Intel 或 macOS-AppleSilicon
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
}

#[test]
fn test_detect_release_platform_linux() {
    // 测试 Linux 平台检测
    // 注意：这个测试只在 Linux 上会通过
    if env::consts::OS == "linux" {
        let platform = detect_release_platform().expect("Should detect Linux platform");

        // Linux 应该是 Linux-x86_64, Linux-x86_64-static, 或 Linux-ARM64
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
}

#[test]
fn test_detect_release_platform_windows() {
    // 测试 Windows 平台检测
    // 注意：这个测试只在 Windows 上会通过
    if env::consts::OS == "windows" {
        let platform = detect_release_platform().expect("Should detect Windows platform");

        // Windows 应该是 Windows-x86_64 或 Windows-ARM64
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
}

#[test]
fn test_detect_release_platform_consistency() {
    // 测试平台检测的一致性
    // 多次调用应该返回相同的结果
    let platform1 = detect_release_platform().expect("Should detect platform");
    let platform2 = detect_release_platform().expect("Should detect platform");
    let platform3 = detect_release_platform().expect("Should detect platform");

    assert_eq!(platform1, platform2);
    assert_eq!(platform2, platform3);
}

#[test]
fn test_detect_release_platform_format_structure() {
    // 测试平台标识符的格式结构
    let platform = detect_release_platform().expect("Should detect platform");

    // 格式应该是：OS-ARCH 或 OS-ARCH-variant
    let parts: Vec<&str> = platform.split('-').collect();
    assert!(
        parts.len() >= 2,
        "Platform format should have at least 2 parts separated by '-', got: {}",
        platform
    );

    // 第一部分应该是操作系统名称
    let os_part = parts[0];
    assert!(
        os_part == "macOS" || os_part == "Linux" || os_part == "Windows",
        "OS part should be macOS, Linux, or Windows, got: {}",
        os_part
    );
}

#[test]
fn test_detect_release_platform_architecture_consistency() {
    // 测试平台检测的架构一致性
    let platform = detect_release_platform().expect("Should detect platform");
    let arch = env::consts::ARCH;

    // 验证平台标识符中的架构与系统架构一致
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
}

#[test]
fn test_detect_release_platform_no_panic() {
    // 测试平台检测不会 panic
    // 即使在不支持的平台上，也应该返回错误而不是 panic
    let result = std::panic::catch_unwind(|| detect_release_platform());

    // 不应该 panic
    assert!(result.is_ok(), "detect_release_platform should not panic");
}

#[test]
fn test_detect_release_platform_error_handling() {
    // 测试错误处理
    // 注意：这个测试主要验证函数不会因为意外的系统状态而 panic
    // 在实际支持的平台上，应该成功返回

    let result = detect_release_platform();

    // 在支持的平台上应该成功
    if env::consts::OS == "macos" || env::consts::OS == "linux" || env::consts::OS == "windows" {
        assert!(result.is_ok(), "Should succeed on supported platforms");
    }
}

#[test]
fn test_platform_identifier_case() {
    // 测试平台标识符的大小写格式
    let platform = detect_release_platform().expect("Should detect platform");

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
}
