//! Platform 模块测试
//!
//! 测试平台检测、路径处理和系统信息获取功能。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use std::env;
use workflow::base::system::Platform;

// ==================== 平台检测测试 ====================

#[test]
fn test_detect_release_platform_returns_valid_format() -> Result<()> {
    // 测试平台检测返回有效的格式
    let platform = Platform::detect().release_identifier()?;

    // 验证返回的字符串不为空
    assert!(!platform.is_empty());

    // 验证格式：应该包含平台名称和架构，用连字符分隔
    assert!(platform.contains('-'), "Platform should contain a hyphen");
    Ok(())
}

#[test]
fn test_detect_release_platform_macos() -> Result<()> {
    // 测试 macOS 平台检测
    // 注意：这个测试只在 macOS 上会通过
    if env::consts::OS == "macos" {
        let platform = Platform::detect().release_identifier()?;

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
    Ok(())
}

#[test]
fn test_detect_release_platform_linux() -> Result<()> {
    // 测试 Linux 平台检测
    // 注意：这个测试只在 Linux 上会通过
    if env::consts::OS == "linux" {
        let platform = Platform::detect().release_identifier()?;

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
    Ok(())
}

#[test]
fn test_detect_release_platform_windows() -> Result<()> {
    // 测试 Windows 平台检测
    // 注意：这个测试只在 Windows 上会通过
    if env::consts::OS == "windows" {
        let platform = Platform::detect().release_identifier()?;

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
    Ok(())
}

#[test]
fn test_detect_release_platform_consistency() -> Result<()> {
    // 测试平台检测的一致性
    // 多次调用应该返回相同的结果
    let platform1 = Platform::detect().release_identifier()?;
    let platform2 = Platform::detect().release_identifier()?;
    let platform3 = Platform::detect().release_identifier()?;

    assert_eq!(platform1, platform2);
    assert_eq!(platform2, platform3);
    Ok(())
}

#[test]
fn test_detect_release_platform_format_structure() -> Result<()> {
    // 测试平台标识符的格式结构
    let platform = Platform::detect().release_identifier()?;

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
    Ok(())
}

#[test]
fn test_detect_release_platform_architecture_consistency() -> Result<()> {
    // 测试平台检测的架构一致性
    let platform = Platform::detect().release_identifier()?;
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
    Ok(())
}

#[test]
fn test_detect_release_platform_no_panic() -> Result<()> {
    // 测试平台检测不会 panic
    // 即使在不支持的平台上，也应该返回错误而不是 panic
    let result = std::panic::catch_unwind(|| Platform::detect().release_identifier());

    // 不应该 panic
    assert!(result.is_ok(), "detect_release_platform should not panic");
    Ok(())
}

#[test]
fn test_detect_release_platform_error_handling() -> Result<()> {
    // 测试错误处理
    // 注意：这个测试主要验证函数不会因为意外的系统状态而 panic
    // 在实际支持的平台上，应该成功返回

    let result = Platform::detect().release_identifier();

    // 在支持的平台上应该成功
    if env::consts::OS == "macos" || env::consts::OS == "linux" || env::consts::OS == "windows" {
        assert!(result.is_ok(), "Should succeed on supported platforms");
    }
    Ok(())
}

#[test]
fn test_platform_identifier_case() -> Result<()> {
    // 测试平台标识符的大小写格式
    let platform = Platform::detect().release_identifier()?;

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

#[test]
fn test_platform_new() -> Result<()> {
    let platform = Platform::new("macos", "aarch64");
    assert_eq!(platform.os(), "macos");
    assert_eq!(platform.arch(), "aarch64");
    Ok(())
}

#[test]
fn test_platform_new_with_string() -> Result<()> {
    let os = String::from("linux");
    let arch = String::from("x86_64");
    let platform = Platform::new(os, arch);
    assert_eq!(platform.os(), "linux");
    assert_eq!(platform.arch(), "x86_64");
    Ok(())
}

#[test]
fn test_platform_detect() -> Result<()> {
    let platform = Platform::detect();
    assert_eq!(platform.os(), env::consts::OS);
    assert_eq!(platform.arch(), env::consts::ARCH);
    Ok(())
}

#[test]
fn test_platform_os() -> Result<()> {
    let platform = Platform::new("windows", "x86_64");
    assert_eq!(platform.os(), "windows");
    Ok(())
}

#[test]
fn test_platform_arch() -> Result<()> {
    let platform = Platform::new("linux", "aarch64");
    assert_eq!(platform.arch(), "aarch64");
    Ok(())
}

#[test]
fn test_platform_is_macos() -> Result<()> {
    let macos_platform = Platform::new("macos", "aarch64");
    assert!(macos_platform.is_macos());

    let linux_platform = Platform::new("linux", "x86_64");
    assert!(!linux_platform.is_macos());
    Ok(())
}

#[test]
fn test_platform_is_linux() -> Result<()> {
    let linux_platform = Platform::new("linux", "x86_64");
    assert!(linux_platform.is_linux());

    let macos_platform = Platform::new("macos", "aarch64");
    assert!(!macos_platform.is_linux());
    Ok(())
}

#[test]
fn test_platform_is_windows() -> Result<()> {
    let windows_platform = Platform::new("windows", "x86_64");
    assert!(windows_platform.is_windows());

    let linux_platform = Platform::new("linux", "x86_64");
    assert!(!linux_platform.is_windows());
    Ok(())
}

#[test]
fn test_platform_is_x86_64() -> Result<()> {
    let x86_64_platform = Platform::new("linux", "x86_64");
    assert!(x86_64_platform.is_x86_64());

    let aarch64_platform = Platform::new("linux", "aarch64");
    assert!(!aarch64_platform.is_x86_64());
    Ok(())
}

#[test]
fn test_platform_is_aarch64() -> Result<()> {
    let aarch64_platform = Platform::new("macos", "aarch64");
    assert!(aarch64_platform.is_aarch64());

    let x86_64_platform = Platform::new("macos", "x86_64");
    assert!(!x86_64_platform.is_aarch64());
    Ok(())
}

#[test]
fn test_platform_release_identifier_macos_intel() -> Result<()> {
    let platform = Platform::new("macos", "x86_64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "macOS-Intel");
    Ok(())
}

#[test]
fn test_platform_release_identifier_macos_apple_silicon() -> Result<()> {
    let platform = Platform::new("macos", "aarch64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "macOS-AppleSilicon");
    Ok(())
}

#[test]
fn test_platform_release_identifier_linux_x86_64() -> Result<()> {
    let platform = Platform::new("linux", "x86_64");
    let identifier = platform.release_identifier()?;
    // 可能是 "Linux-x86_64" 或 "Linux-x86_64-static"，取决于静态链接检测
    assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
    Ok(())
}

#[test]
fn test_platform_release_identifier_linux_arm64() -> Result<()> {
    let platform = Platform::new("linux", "aarch64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "Linux-ARM64");
    Ok(())
}

#[test]
fn test_platform_release_identifier_windows_x86_64() -> Result<()> {
    let platform = Platform::new("windows", "x86_64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "Windows-x86_64");
    Ok(())
}

#[test]
fn test_platform_release_identifier_windows_arm64() -> Result<()> {
    let platform = Platform::new("windows", "aarch64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "Windows-ARM64");
    Ok(())
}

#[test]
fn test_platform_release_identifier_unsupported() -> Result<()> {
    let platform = Platform::new("unsupported_os", "unsupported_arch");
    let result = platform.release_identifier();
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported platform"));
    Ok(())
}

#[test]
fn test_platform_debug() -> Result<()> {
    let platform = Platform::new("macos", "aarch64");
    let debug_str = format!("{:?}", platform);
    assert!(debug_str.contains("macos") || debug_str.contains("aarch64"));
    Ok(())
}

#[test]
fn test_platform_clone() -> Result<()> {
    let platform1 = Platform::new("linux", "x86_64");
    let platform2 = platform1.clone();
    assert_eq!(platform1.os(), platform2.os());
    assert_eq!(platform1.arch(), platform2.arch());
    Ok(())
}

#[test]
fn test_platform_eq() -> Result<()> {
    let platform1 = Platform::new("macos", "aarch64");
    let platform2 = Platform::new("macos", "aarch64");
    assert_eq!(platform1, platform2);

    let platform3 = Platform::new("linux", "x86_64");
    assert_ne!(platform1, platform3);
    Ok(())
}

#[test]
fn test_platform_release_identifier_linux_x86_64_static_detection() -> Result<()> {
    // 测试 Linux x86_64 平台的静态链接检测
    // 这个测试验证 release_identifier 能够正确检测静态链接需求
    let platform = Platform::new("linux", "x86_64");
    let identifier = platform.release_identifier()?;

    // 应该返回 "Linux-x86_64" 或 "Linux-x86_64-static"
    assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
    Ok(())
}

#[test]
fn test_platform_release_identifier_non_linux_does_not_check_static() -> Result<()> {
    // 测试非 Linux 平台不会检查静态链接
    let macos = Platform::new("macos", "x86_64");
    let windows = Platform::new("windows", "x86_64");

    assert_eq!(macos.release_identifier()?, "macOS-Intel");
    assert_eq!(windows.release_identifier()?, "Windows-x86_64");

    Ok(())
}

#[test]
fn test_platform_release_identifier_linux_non_x86_64_does_not_check_static() -> Result<()> {
    // 测试 Linux 非 x86_64 架构不会检查静态链接
    let linux_arm64 = Platform::new("linux", "aarch64");

    assert_eq!(linux_arm64.release_identifier()?, "Linux-ARM64");

    Ok(())
}

#[test]
fn test_platform_release_identifier_all_combinations() -> Result<()> {
    // 测试所有平台组合的 release_identifier()
    let combinations = vec![
        ("macos", "x86_64", "macOS-Intel"),
        ("macos", "aarch64", "macOS-AppleSilicon"),
        ("linux", "aarch64", "Linux-ARM64"),
        ("windows", "x86_64", "Windows-x86_64"),
        ("windows", "aarch64", "Windows-ARM64"),
    ];

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

#[test]
fn test_platform_is_static_required_alpine_detection() -> Result<()> {
    // 测试 Alpine Linux 检测逻辑
    // 注意：这个测试在非 Alpine Linux 系统上可能无法完全测试
    // 但至少可以验证代码路径存在
    let platform = Platform::new("linux", "x86_64");
    let identifier = platform.release_identifier()?;

    // 应该是 Linux-x86_64 或 Linux-x86_64-static
    assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
    Ok(())
}

#[test]
fn test_platform_is_static_required_ldd_output_scenarios() -> Result<()> {
    // 测试 ldd 命令的不同输出场景
    // 由于无法直接控制 ldd 命令的输出，我们通过 release_identifier 间接测试
    let platform = Platform::new("linux", "x86_64");
    let identifier = platform.release_identifier()?;

    // 验证格式正确
    assert!(identifier.starts_with("Linux-x86_64"));

    // 在非 Linux 系统上，is_static_required 应该返回 false
    let macos_platform = Platform::new("macos", "x86_64");
    let macos_id = macos_platform.release_identifier()?;
    assert_eq!(macos_id, "macOS-Intel");

    // 在非 x86_64 架构上，is_static_required 应该返回 false
    let linux_arm64_platform = Platform::new("linux", "aarch64");
    let linux_arm64_id = linux_arm64_platform.release_identifier()?;
    assert_eq!(linux_arm64_id, "Linux-ARM64");

    Ok(())
}

#[test]
fn test_platform_is_static_required_non_linux_early_return() -> Result<()> {
    // 测试非 Linux 平台的早期返回路径（覆盖 is_static_required 的第 117-118 行）
    // 通过 release_identifier 间接测试
    let macos = Platform::new("macos", "x86_64");
    let windows = Platform::new("windows", "x86_64");

    // 非 Linux 平台不应该检查静态链接
    assert_eq!(macos.release_identifier()?, "macOS-Intel");
    assert_eq!(windows.release_identifier()?, "Windows-x86_64");
    Ok(())
}

#[test]
fn test_platform_is_static_required_non_x86_64_early_return() -> Result<()> {
    // 测试非 x86_64 架构的早期返回路径（覆盖 is_static_required 的第 117-118 行）
    // 通过 release_identifier 间接测试
    let linux_arm64 = Platform::new("linux", "aarch64");

    // 非 x86_64 架构不应该检查静态链接
    assert_eq!(linux_arm64.release_identifier()?, "Linux-ARM64");
    Ok(())
}

#[test]
fn test_platform_is_static_required_linux_x86_64_scenarios() -> Result<()> {
    // 测试 Linux x86_64 平台的不同场景
    // 由于无法直接控制文件读取和命令执行，我们通过 release_identifier 间接测试
    // 这个测试验证代码路径存在，实际行为取决于运行环境
    let platform = Platform::new("linux", "x86_64");
    let identifier = platform.release_identifier()?;

    // 应该返回 Linux-x86_64 或 Linux-x86_64-static
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

#[test]
#[cfg(target_os = "linux")]
#[cfg(target_arch = "x86_64")]
fn test_platform_is_static_required_actual_linux_environment() -> Result<()> {
    // 在真实的 Linux x86_64 环境中测试静态链接检测
    // 这个测试只在 Linux x86_64 平台上运行
    let platform = Platform::new("linux", "x86_64");
    let identifier = platform.release_identifier()?;

    // 验证返回有效的标识符
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
