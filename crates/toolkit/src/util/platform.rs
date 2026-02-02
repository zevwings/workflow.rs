//! 平台检测工具模块
//!
//! 提供平台检测相关的工具函数，用于识别当前运行的操作系统和架构。

use std::env;
use std::process::Command;

use thiserror::Error;

use crate::util::fs::FileReader;

/// 平台操作错误
#[derive(Debug, Error)]
pub enum PlatformError {
    /// 平台操作错误
    #[error("Platform error: {0}")]
    Operation(String),
}

/// 平台信息结构体
///
/// 封装操作系统和架构信息，提供平台检测和标识符生成功能。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    /// 操作系统类型
    os: String,
    /// 系统架构
    arch: String,
}

impl Platform {
    /// 创建新的平台实例
    ///
    /// # 参数
    ///
    /// * `os` - 操作系统类型（如 "macos", "linux", "windows"）
    /// * `arch` - 系统架构（如 "x86_64", "aarch64"）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use toolkit::Platform;
    ///
    /// let platform = Platform::new("macos", "aarch64");
    /// ```
    pub fn new(os: impl Into<String>, arch: impl Into<String>) -> Self {
        Self {
            os: os.into(),
            arch: arch.into(),
        }
    }

    /// 检测当前运行平台
    ///
    /// 自动检测当前系统的操作系统和架构信息。
    ///
    /// # 返回
    ///
    /// 返回当前平台的 `Platform` 实例。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use toolkit::Platform;
    ///
    /// let platform = Platform::detect();
    /// println!("Current platform: {} {}", platform.os(), platform.arch());
    /// ```
    pub fn detect() -> Self {
        Self::new(env::consts::OS, env::consts::ARCH)
    }

    /// 获取操作系统类型
    ///
    /// # 返回
    ///
    /// 返回操作系统类型字符串（如 "macos", "linux", "windows"）。
    pub fn os(&self) -> &str {
        &self.os
    }

    /// 获取系统架构
    ///
    /// # 返回
    ///
    /// 返回系统架构字符串（如 "x86_64", "aarch64"）。
    pub fn arch(&self) -> &str {
        &self.arch
    }

    /// 检查是否为 macOS 平台
    pub fn is_macos(&self) -> bool {
        self.os == "macos"
    }

    /// 检查是否为 Linux 平台
    pub fn is_linux(&self) -> bool {
        self.os == "linux"
    }

    /// 检查是否为 Windows 平台
    pub fn is_windows(&self) -> bool {
        self.os == "windows"
    }

    /// 检查是否为 x86_64 架构
    pub fn is_x86_64(&self) -> bool {
        self.arch == "x86_64"
    }

    /// 检查是否为 ARM64/aarch64 架构
    pub fn is_aarch64(&self) -> bool {
        self.arch == "aarch64"
    }

    /// 检测 Linux x86_64 是否需要静态链接版本
    ///
    /// 通过以下方法检测：
    /// 1. 检查是否是 Alpine Linux（通常使用 musl）
    /// 2. 检测当前二进制是否静态链接（使用 `ldd` 命令）
    ///
    /// # 返回
    ///
    /// 如果需要静态链接版本返回 `true`，否则返回 `false`。
    fn is_static_required(&self) -> bool {
        if !self.is_linux() || !self.is_x86_64() {
            return false;
        }

        // 方法1: 检查是否是 Alpine Linux（通常使用 musl）
        if let Ok(os_release) = FileReader::new("/etc/os-release").to_string() {
            if os_release.contains("Alpine") || os_release.contains("ID=alpine") {
                return true;
            }
        }

        // 方法2: 尝试检测当前二进制是否静态链接
        // 如果 ldd 命令失败或没有输出，可能是静态链接
        if let Ok(output) = Command::new("ldd")
            .arg(env::current_exe().unwrap_or_default())
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // 如果 ldd 输出 "not a dynamic executable" 或 "statically linked"
            // 说明是静态链接，应该使用 static 版本
            if output_str.contains("not a dynamic executable")
                || output_str.contains("statically linked")
                || output_str.is_empty()
            {
                return true;
            }
        } else {
            // ldd 命令不存在或失败，可能是 musl 环境（Alpine 等）
            // 在这种情况下，尝试使用 static 版本
            return true;
        }

        false
    }

    /// 生成 GitHub Releases 格式的平台标识符
    ///
    /// 返回平台标识符字符串，用于匹配 GitHub Releases 中的资源文件。
    /// 支持的平台格式：
    /// - macOS: `macOS-Intel`, `macOS-AppleSilicon`
    /// - Linux: `Linux-x86_64`, `Linux-x86_64-static`, `Linux-ARM64`
    /// - Windows: `Windows-x86_64`, `Windows-ARM64`
    ///
    /// # 返回
    ///
    /// 返回平台标识符字符串。
    ///
    /// # 错误
    ///
    /// 如果平台不支持，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use toolkit::Platform;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let platform = Platform::detect();
    /// let identifier = platform.release_identifier()?;
    /// println!("Release identifier: {}", identifier);
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_identifier(&self) -> Result<String, PlatformError> {
        match (self.os.as_str(), self.arch.as_str()) {
            ("macos", "x86_64") => Ok("macOS-Intel".to_string()),
            ("macos", "aarch64") => Ok("macOS-AppleSilicon".to_string()),
            ("linux", "x86_64") => {
                if self.is_static_required() {
                    Ok("Linux-x86_64-static".to_string())
                } else {
                    Ok("Linux-x86_64".to_string())
                }
            }
            ("linux", "aarch64") => Ok("Linux-ARM64".to_string()),
            ("windows", "x86_64") => Ok("Windows-x86_64".to_string()),
            ("windows", "aarch64") => Ok("Windows-ARM64".to_string()),
            _ => Err(PlatformError::Operation(format!(
                "Unsupported platform: {}-{}",
                self.os, self.arch
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    // ==================== 平台检测测试 ====================

    #[test]
    fn test_platform_detect_release_identifier_returns_valid_format() {
        // 测试平台检测返回有效的格式
        let platform = Platform::detect()
            .release_identifier()
            .expect("Should detect platform");

        // 验证返回的字符串不为空
        assert!(!platform.is_empty());

        // 验证格式：应该包含平台名称和架构，用连字符分隔
        assert!(platform.contains('-'), "Platform should contain a hyphen");
    }

    #[test]
    fn test_platform_detect_release_identifier_macos() {
        // 测试 macOS 平台检测
        // 注意：这个测试只在 macOS 上会通过
        if env::consts::OS == "macos" {
            let platform = Platform::detect()
                .release_identifier()
                .expect("Should detect macOS platform");

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
    fn test_platform_detect_release_identifier_linux() {
        // 测试 Linux 平台检测
        // 注意：这个测试只在 Linux 上会通过
        if env::consts::OS == "linux" {
            let platform = Platform::detect()
                .release_identifier()
                .expect("Should detect Linux platform");

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
    fn test_platform_detect_release_identifier_windows() {
        // 测试 Windows 平台检测
        // 注意：这个测试只在 Windows 上会通过
        if env::consts::OS == "windows" {
            let platform = Platform::detect()
                .release_identifier()
                .expect("Should detect Windows platform");

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
    fn test_platform_detect_release_identifier_consistency() {
        // 测试平台检测的一致性
        // 多次调用应该返回相同的结果
        let platform1 = Platform::detect()
            .release_identifier()
            .expect("Should detect platform");
        let platform2 = Platform::detect()
            .release_identifier()
            .expect("Should detect platform");
        let platform3 = Platform::detect()
            .release_identifier()
            .expect("Should detect platform");

        assert_eq!(platform1, platform2);
        assert_eq!(platform2, platform3);
    }

    #[test]
    fn test_platform_detect_release_identifier_format_structure() {
        // 测试平台标识符的格式结构
        let platform = Platform::detect()
            .release_identifier()
            .expect("Should detect platform");

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
    fn test_platform_detect_release_identifier_architecture_consistency() {
        // 测试平台检测的架构一致性
        let platform = Platform::detect()
            .release_identifier()
            .expect("Should detect platform");
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
    fn test_platform_detect_release_identifier_no_panic() {
        // 测试平台检测不会 panic
        // 即使在不支持的平台上，也应该返回错误而不是 panic
        let result = std::panic::catch_unwind(|| Platform::detect().release_identifier());

        // 不应该 panic
        assert!(
            result.is_ok(),
            "Platform::detect().release_identifier() should not panic"
        );
    }

    #[test]
    fn test_platform_detect_release_identifier_error_handling() {
        // 测试错误处理
        // 注意：这个测试主要验证函数不会因为意外的系统状态而 panic
        // 在实际支持的平台上，应该成功返回

        let result = Platform::detect().release_identifier();

        // 在支持的平台上应该成功
        if env::consts::OS == "macos" || env::consts::OS == "linux" || env::consts::OS == "windows"
        {
            assert!(result.is_ok(), "Should succeed on supported platforms");
        }
    }

    #[test]
    fn test_platform_identifier_case() {
        // 测试平台标识符的大小写格式
        let platform = Platform::detect()
            .release_identifier()
            .expect("Should detect platform");

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

    // ==================== Platform::release_identifier 测试 ====================

    #[test]
    fn test_platform_release_identifier_linux_x86_64() {
        // 测试 Linux x86_64 平台标识符
        // 注意：Linux x86_64 可能是 "Linux-x86_64" 或 "Linux-x86_64-static"
        // 取决于 is_static_required() 的结果
        let platform = Platform::new("linux", "x86_64");
        let identifier = platform
            .release_identifier()
            .expect("Should return identifier");

        assert!(
            identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static",
            "Linux x86_64 should be Linux-x86_64 or Linux-x86_64-static, got: {}",
            identifier
        );
    }

    // 使用参数化测试简化支持的平台组合测试
    #[rstest]
    #[case("macos", "x86_64", "macOS-Intel")]
    #[case("macos", "aarch64", "macOS-AppleSilicon")]
    #[case("linux", "aarch64", "Linux-ARM64")]
    #[case("windows", "x86_64", "Windows-x86_64")]
    #[case("windows", "aarch64", "Windows-ARM64")]
    fn test_platform_release_identifier_supported(
        #[case] os: &str,
        #[case] arch: &str,
        #[case] expected: &str,
    ) {
        let platform = Platform::new(os, arch);
        let identifier = platform
            .release_identifier()
            .unwrap_or_else(|_| panic!("Should return identifier for {}-{}", os, arch));
        assert_eq!(identifier, expected);
    }

    #[rstest]
    #[case(
        "unsupported",
        "unknown",
        Some("Unsupported platform"),
        Some("unsupported-unknown")
    )]
    #[case("freebsd", "x86_64", None, None)]
    #[case("macos", "armv7", None, None)]
    fn test_platform_release_identifier_unsupported(
        #[case] os: &str,
        #[case] arch: &str,
        #[case] expected_error_contains: Option<&str>,
        #[case] expected_error_contains2: Option<&str>,
    ) {
        let platform = Platform::new(os, arch);
        let result = platform.release_identifier();

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        if let Some(expected) = expected_error_contains {
            assert!(error_msg.contains(expected));
        }
        if let Some(expected) = expected_error_contains2 {
            assert!(error_msg.contains(expected));
        }
    }

    #[test]
    fn test_platform_release_identifier_consistency() {
        // 测试相同平台多次调用的一致性
        let platform = Platform::new("macos", "aarch64");

        let id1 = platform
            .release_identifier()
            .expect("Should return identifier");
        let id2 = platform
            .release_identifier()
            .expect("Should return identifier");
        let id3 = platform
            .release_identifier()
            .expect("Should return identifier");

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
    }
}
