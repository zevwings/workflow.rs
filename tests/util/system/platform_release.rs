//! Base Util Platform Release 模块测试
//!
//! 测试平台检测工具中 release_identifier 相关的功能。

use pretty_assertions::assert_eq;
use workflow::base::util::platform::Platform;

#[test]
fn test_platform_release_identifier_macos_intel() -> color_eyre::Result<()> {
    let platform = Platform::new("macos", "x86_64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "macOS-Intel");
    Ok(())
}

#[test]
fn test_platform_release_identifier_macos_apple_silicon() -> color_eyre::Result<()> {
    let platform = Platform::new("macos", "aarch64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "macOS-AppleSilicon");
    Ok(())
}

#[test]
fn test_platform_release_identifier_linux_x86_64() -> color_eyre::Result<()> {
    let platform = Platform::new("linux", "x86_64");
    let identifier = platform.release_identifier()?;
    // 可能是 "Linux-x86_64" 或 "Linux-x86_64-static"，取决于静态链接检测
    assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
    Ok(())
}

#[test]
fn test_platform_release_identifier_linux_arm64() -> color_eyre::Result<()> {
    let platform = Platform::new("linux", "aarch64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "Linux-ARM64");
    Ok(())
}

#[test]
fn test_platform_release_identifier_windows_x86_64() -> color_eyre::Result<()> {
    let platform = Platform::new("windows", "x86_64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "Windows-x86_64");
    Ok(())
}

#[test]
fn test_platform_release_identifier_windows_arm64() -> color_eyre::Result<()> {
    let platform = Platform::new("windows", "aarch64");
    let identifier = platform.release_identifier()?;
    assert_eq!(identifier, "Windows-ARM64");
    Ok(())
}

#[test]
fn test_platform_release_identifier_unsupported() {
    let platform = Platform::new("unsupported_os", "unsupported_arch");
    let result = platform.release_identifier();
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported platform"));
}
