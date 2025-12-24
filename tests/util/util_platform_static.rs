//! Base Util Platform Static 模块测试
//!
//! 测试平台检测工具中静态链接检测相关的功能。

use workflow::base::util::platform::Platform;

#[test]
fn test_platform_release_identifier_linux_x86_64_static_detection() {
    // 测试 Linux x86_64 平台的静态链接检测
    // 这个测试验证 release_identifier 能够正确检测静态链接需求
    let platform = Platform::new("linux", "x86_64");
    let result = platform.release_identifier();

    // 应该返回 "Linux-x86_64" 或 "Linux-x86_64-static"
    assert!(result.is_ok());
    let identifier = result.unwrap();
    assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
}

#[test]
fn test_platform_release_identifier_non_linux_does_not_check_static() -> color_eyre::Result<()> {
    // 测试非 Linux 平台不会检查静态链接
    let macos = Platform::new("macos", "x86_64");
    let windows = Platform::new("windows", "x86_64");

    assert_eq!(macos.release_identifier()?, "macOS-Intel");
    assert_eq!(windows.release_identifier()?, "Windows-x86_64");

    Ok(())
}

#[test]
fn test_platform_release_identifier_linux_non_x86_64_does_not_check_static() -> color_eyre::Result<()> {
    // 测试 Linux 非 x86_64 架构不会检查静态链接
    let linux_arm64 = Platform::new("linux", "aarch64");

    assert_eq!(linux_arm64.release_identifier()?, "Linux-ARM64");

    Ok(())
}

