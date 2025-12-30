//! 平台检测工具模块
//!
//! 提供平台检测相关的工具函数，用于识别当前运行的操作系统和架构。

use color_eyre::Result;
use std::env;
use std::process::Command;

use crate::base::fs::file::FileReader;

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
    /// use workflow::base::system::Platform;
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
    /// use workflow::base::system::Platform;
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
        if let Ok(output) = Command::new("ldd").arg(env::current_exe().unwrap_or_default()).output()
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
    /// use workflow::base::system::Platform;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let platform = Platform::detect();
    /// let identifier = platform.release_identifier()?;
    /// println!("Release identifier: {}", identifier);
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_identifier(&self) -> Result<String> {
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
            _ => color_eyre::eyre::bail!("Unsupported platform: {}-{}", self.os, self.arch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    // ==================== Platform Detection Tests ====================

    /// 测试检测平台并获取有效的发布标识符格式
    ///
    /// ## 测试目的
    /// 验证 Platform::detect().release_identifier() 能够正确检测当前平台并返回有效的发布标识符。
    ///
    /// ## 测试场景
    /// 1. 调用 detect() 方法检测平台
    /// 2. 调用 release_identifier() 获取发布标识符
    /// 3. 验证标识符不为空且包含连字符
    ///
    /// ## 预期结果
    /// - 成功返回平台标识符
    /// - 标识符不为空
    /// - 标识符包含连字符（格式：OS-ARCH）
    #[test]
    fn test_detect_release_platform_return_ok() -> Result<()> {
        let platform = Platform::detect().release_identifier()?;
        assert!(!platform.is_empty());
        assert!(platform.contains('-'), "Platform should contain a hyphen");
        Ok(())
    }

    /// 测试在 macOS 上检测平台返回 macOS 标识符
    ///
    /// ## 测试目的
    /// 验证在 macOS 系统上，Platform::detect().release_identifier() 能够返回正确的 macOS 平台标识符。
    ///
    /// ## 测试场景
    /// 1. 检查当前系统是否为 macOS
    /// 2. 如果是 macOS，检测平台并获取标识符
    /// 3. 验证标识符为 "macOS-Intel" 或 "macOS-AppleSilicon"
    /// 4. 根据架构验证具体标识符
    ///
    /// ## 预期结果
    /// - x86_64 架构返回 "macOS-Intel"
    /// - aarch64 架构返回 "macOS-AppleSilicon"
    ///
    /// ## 注意
    /// 此测试仅在 macOS 系统上执行。
    #[test]
    fn test_detect_release_platform_on_macos_return_ok() -> Result<()> {
        if env::consts::OS == "macos" {
            let platform = Platform::detect().release_identifier()?;
            assert!(
                platform == "macOS-Intel" || platform == "macOS-AppleSilicon",
                "macOS platform should be macOS-Intel or macOS-AppleSilicon, got: {}",
                platform
            );
            if env::consts::ARCH == "x86_64" {
                assert_eq!(platform, "macOS-Intel");
            } else if env::consts::ARCH == "aarch64" {
                assert_eq!(platform, "macOS-AppleSilicon");
            }
        }
        Ok(())
    }

    /// 测试在 Linux 上检测平台返回 Linux 标识符
    ///
    /// ## 测试目的
    /// 验证在 Linux 系统上，Platform::detect().release_identifier() 能够返回正确的 Linux 平台标识符。
    ///
    /// ## 测试场景
    /// 1. 检查当前系统是否为 Linux
    /// 2. 如果是 Linux，检测平台并获取标识符
    /// 3. 验证标识符为 "Linux-x86_64"、"Linux-x86_64-static" 或 "Linux-ARM64"
    /// 4. 根据架构验证具体标识符
    ///
    /// ## 预期结果
    /// - x86_64 架构返回 "Linux-x86_64" 或 "Linux-x86_64-static"
    /// - aarch64 架构返回 "Linux-ARM64"
    ///
    /// ## 注意
    /// 此测试仅在 Linux 系统上执行。
    #[test]
    fn test_detect_release_platform_on_linux_return_ok() -> Result<()> {
        if env::consts::OS == "linux" {
            let platform = Platform::detect().release_identifier()?;
            assert!(
                platform == "Linux-x86_64"
                    || platform == "Linux-x86_64-static"
                    || platform == "Linux-ARM64",
                "Linux platform should be Linux-x86_64, Linux-x86_64-static, or Linux-ARM64, got: {}",
                platform
            );
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

    /// 测试在 Windows 上检测平台返回 Windows 标识符
    ///
    /// ## 测试目的
    /// 验证在 Windows 系统上，Platform::detect().release_identifier() 能够返回正确的 Windows 平台标识符。
    ///
    /// ## 测试场景
    /// 1. 检查当前系统是否为 Windows
    /// 2. 如果是 Windows，检测平台并获取标识符
    /// 3. 验证标识符为 "Windows-x86_64" 或 "Windows-ARM64"
    /// 4. 根据架构验证具体标识符
    ///
    /// ## 预期结果
    /// - x86_64 架构返回 "Windows-x86_64"
    /// - aarch64 架构返回 "Windows-ARM64"
    ///
    /// ## 注意
    /// 此测试仅在 Windows 系统上执行。
    #[test]
    fn test_detect_release_platform_on_windows_return_ok() -> Result<()> {
        if env::consts::OS == "windows" {
            let platform = Platform::detect().release_identifier()?;
            assert!(
                platform == "Windows-x86_64" || platform == "Windows-ARM64",
                "Windows platform should be Windows-x86_64 or Windows-ARM64, got: {}",
                platform
            );
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
    /// 验证 Platform::detect().release_identifier() 在多次调用时返回一致的结果，确保检测的稳定性和一致性。
    ///
    /// ## 测试场景
    /// 1. 连续三次调用 detect().release_identifier()
    /// 2. 比较三次调用的结果
    /// 3. 验证所有结果都相同
    ///
    /// ## 预期结果
    /// - 三次调用的结果完全一致
    /// - 平台检测结果稳定可靠
    #[test]
    fn test_detect_release_platform_with_multiple_calls_return_collect() -> Result<()> {
        let platform1 = Platform::detect().release_identifier()?;
        let platform2 = Platform::detect().release_identifier()?;
        let platform3 = Platform::detect().release_identifier()?;
        assert_eq!(platform1, platform2);
        assert_eq!(platform2, platform3);
        Ok(())
    }

    /// 测试检测平台返回结构化的标识符格式（OS-ARCH 格式）
    ///
    /// ## 测试目的
    /// 验证 Platform::detect().release_identifier() 返回的标识符符合预期的结构化格式（OS-ARCH）。
    ///
    /// ## 测试场景
    /// 1. 检测平台并获取标识符
    /// 2. 使用连字符分割标识符
    /// 3. 验证分割后的部分数量至少为 2（OS 和 ARCH）
    ///
    /// ## 预期结果
    /// - 标识符格式为 "OS-ARCH" 或 "OS-ARCH-*"
    /// - 至少包含操作系统和架构两部分
    #[test]
    fn test_detect_release_platform_with_valid_format_return_ok() -> Result<()> {
        let platform = Platform::detect().release_identifier()?;
        let parts: Vec<&str> = platform.split('-').collect();
        assert!(
            parts.len() >= 2,
            "Platform format should have at least 2 parts separated by '-', got: {}",
            platform
        );
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
    /// 验证 Platform::detect().release_identifier() 返回的标识符与当前系统的架构信息匹配。
    ///
    /// ## 测试场景
    /// 1. 获取当前系统架构
    /// 2. 检测平台并获取标识符
    /// 3. 验证标识符包含与架构匹配的关键词
    ///
    /// ## 预期结果
    /// - x86_64 架构的标识符包含 "x86_64" 或 "Intel"
    /// - aarch64 架构的标识符包含 "ARM64" 或 "AppleSilicon"
    #[test]
    fn test_detect_release_platform_with_system_arch_return_ok() -> Result<()> {
        let arch = env::consts::ARCH;
        let platform = Platform::detect().release_identifier()?;
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

    /// 测试在任何平台上检测都不会 panic
    ///
    /// ## 测试目的
    /// 验证 Platform::detect().release_identifier() 在任何平台上都不会产生 panic，确保错误处理的安全性。
    ///
    /// ## 测试场景
    /// 1. 使用 catch_unwind 捕获可能的 panic
    /// 2. 调用 detect().release_identifier()
    /// 3. 验证没有发生 panic
    ///
    /// ## 预期结果
    /// - 方法执行不产生 panic
    /// - 错误通过 Result 类型返回，而不是 panic
    #[test]
    fn test_detect_release_platform_with_any_platform_does_not_panic_return_ok() -> Result<()> {
        let result = std::panic::catch_unwind(|| Platform::detect().release_identifier());
        assert!(result.is_ok(), "detect_release_platform should not panic");
        Ok(())
    }

    /// 测试在支持的 OS 上检测平台返回成功
    ///
    /// ## 测试目的
    /// 验证 Platform::detect().release_identifier() 在支持的操作系统（macOS、Linux、Windows）上能够成功返回结果。
    ///
    /// ## 测试场景
    /// 1. 检测平台并获取标识符
    /// 2. 检查当前系统是否为支持的操作系统
    /// 3. 如果是支持的 OS，验证返回成功
    ///
    /// ## 预期结果
    /// - 在 macOS、Linux、Windows 上返回成功
    /// - 返回的 Result 为 Ok
    #[test]
    fn test_detect_release_platform_with_supported_os_return_ok() -> Result<()> {
        let result = Platform::detect().release_identifier();
        if env::consts::OS == "macos" || env::consts::OS == "linux" || env::consts::OS == "windows"
        {
            assert!(result.is_ok(), "Should succeed on supported platforms");
        }
        Ok(())
    }

    /// 测试平台标识符的大小写格式正确
    ///
    /// ## 测试目的
    /// 验证 Platform::detect().release_identifier() 返回的标识符中操作系统部分的大小写格式正确。
    ///
    /// ## 测试场景
    /// 1. 检测平台并获取标识符
    /// 2. 检查标识符开头部分的大小写
    /// 3. 验证 macOS、Linux、Windows 的大小写格式正确
    ///
    /// ## 预期结果
    /// - macOS 标识符以 "macOS" 开头
    /// - Linux 标识符以 "Linux" 开头
    /// - Windows 标识符以 "Windows" 开头
    #[test]
    fn test_platform_identifier_with_detected_platform_return_ok() -> Result<()> {
        let platform = Platform::detect().release_identifier()?;
        if platform.starts_with("macOS") {
            assert_eq!(&platform[0..5], "macOS");
        }
        if platform.starts_with("Linux") {
            assert_eq!(&platform[0..5], "Linux");
        }
        if platform.starts_with("Windows") {
            assert_eq!(&platform[0..7], "Windows");
        }
        Ok(())
    }

    // ==================== Platform 结构体基础方法测试 ====================

    /// 测试使用 OS 和架构创建 Platform 实例
    ///
    /// ## 测试目的
    /// 验证 Platform::new() 能够使用操作系统和架构字符串创建 Platform 实例。
    ///
    /// ## 测试场景
    /// 1. 使用 OS 和架构字符串调用 new() 方法
    /// 2. 验证创建的实例的 os() 和 arch() 方法返回正确的值
    ///
    /// ## 预期结果
    /// - 成功创建 Platform 实例
    /// - os() 返回传入的操作系统字符串
    /// - arch() 返回传入的架构字符串
    #[test]
    fn test_platform_new_with_os_and_arch_return_ok() -> Result<()> {
        let platform = Platform::new("macos", "aarch64");
        assert_eq!(platform.os(), "macos");
        assert_eq!(platform.arch(), "aarch64");
        Ok(())
    }

    /// 测试使用 String 参数创建 Platform 实例
    ///
    /// ## 测试目的
    /// 验证 Platform::new() 能够接受 String 类型的参数创建 Platform 实例。
    ///
    /// ## 测试场景
    /// 1. 创建 String 类型的 OS 和架构参数
    /// 2. 使用 String 参数调用 new() 方法
    /// 3. 验证创建的实例的属性正确
    ///
    /// ## 预期结果
    /// - 成功创建 Platform 实例
    /// - os() 和 arch() 返回正确的值
    #[test]
    fn test_platform_new_with_string_params_return_ok() -> Result<()> {
        let os = String::from("linux");
        let arch = String::from("x86_64");
        let platform = Platform::new(os, arch);
        assert_eq!(platform.os(), "linux");
        assert_eq!(platform.arch(), "x86_64");
        Ok(())
    }

    /// 测试检测当前系统平台
    ///
    /// ## 测试目的
    /// 验证 Platform::detect() 能够正确检测当前系统的操作系统和架构信息。
    ///
    /// ## 测试场景
    /// 1. 调用 detect() 方法检测当前系统
    /// 2. 获取检测到的平台实例
    /// 3. 验证 os() 和 arch() 与系统常量匹配
    ///
    /// ## 预期结果
    /// - os() 返回 env::consts::OS
    /// - arch() 返回 env::consts::ARCH
    #[test]
    fn test_platform_detect_with_system_info_return_ok() -> Result<()> {
        let platform = Platform::detect();
        assert_eq!(platform.os(), env::consts::OS);
        assert_eq!(platform.arch(), env::consts::ARCH);
        Ok(())
    }

    /// 测试获取 Platform 实例的操作系统
    ///
    /// ## 测试目的
    /// 验证 Platform::os() 能够正确返回 Platform 实例的操作系统信息。
    ///
    /// ## 测试场景
    /// 1. 创建指定操作系统的 Platform 实例
    /// 2. 调用 os() 方法获取操作系统
    /// 3. 验证返回的操作系统与创建时传入的值一致
    ///
    /// ## 预期结果
    /// - os() 返回创建时传入的操作系统字符串
    #[test]
    fn test_platform_os_with_platform_instance_return_ok() -> Result<()> {
        let platform = Platform::new("windows", "x86_64");
        assert_eq!(platform.os(), "windows");
        Ok(())
    }

    /// 测试获取 Platform 实例的架构
    ///
    /// ## 测试目的
    /// 验证 Platform::arch() 能够正确返回 Platform 实例的架构信息。
    ///
    /// ## 测试场景
    /// 1. 创建指定架构的 Platform 实例
    /// 2. 调用 arch() 方法获取架构
    /// 3. 验证返回的架构与创建时传入的值一致
    ///
    /// ## 预期结果
    /// - arch() 返回创建时传入的架构字符串
    #[test]
    fn test_platform_arch_with_platform_instance_return_ok() -> Result<()> {
        let platform = Platform::new("linux", "aarch64");
        assert_eq!(platform.arch(), "aarch64");
        Ok(())
    }

    /// 测试判断 Platform 是否为特定操作系统（参数化测试）
    ///
    /// ## 测试目的
    /// 验证 Platform 的 is_macos()、is_linux()、is_windows() 方法能够正确判断平台的操作系统类型。
    ///
    /// ## 测试场景
    /// 1. 创建不同操作系统的 Platform 实例
    /// 2. 调用对应的判断方法（is_macos、is_linux、is_windows）
    /// 3. 验证判断结果正确
    ///
    /// ## 预期结果
    /// - 匹配的操作系统返回 true
    /// - 不匹配的操作系统返回 false
    #[rstest]
    #[case("macos", "aarch64", "linux", "x86_64", true, false)]
    #[case("linux", "x86_64", "macos", "aarch64", true, false)]
    #[case("windows", "x86_64", "linux", "x86_64", true, false)]
    fn test_platform_is_os_return_ok(
        #[case] target_os: &str,
        #[case] target_arch: &str,
        #[case] other_os: &str,
        #[case] other_arch: &str,
        #[case] target_should_be_true: bool,
        #[case] other_should_be_true: bool,
    ) -> Result<()> {
        let target_platform = Platform::new(target_os, target_arch);
        let other_platform = Platform::new(other_os, other_arch);
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

    /// 测试判断 Platform 是否为特定架构（参数化测试）
    ///
    /// ## 测试目的
    /// 验证 Platform 的 is_x86_64()、is_aarch64() 方法能够正确判断平台的架构类型。
    ///
    /// ## 测试场景
    /// 1. 创建不同架构的 Platform 实例
    /// 2. 调用对应的判断方法（is_x86_64、is_aarch64）
    /// 3. 验证判断结果正确
    ///
    /// ## 预期结果
    /// - 匹配的架构返回 true
    /// - 不匹配的架构返回 false
    #[rstest]
    #[case("linux", "x86_64", "linux", "aarch64", true, false)]
    #[case("macos", "aarch64", "macos", "x86_64", true, false)]
    fn test_platform_is_arch_return_ok(
        #[case] target_os: &str,
        #[case] target_arch: &str,
        #[case] other_os: &str,
        #[case] other_arch: &str,
        #[case] target_should_be_true: bool,
        #[case] other_should_be_true: bool,
    ) -> Result<()> {
        let target_platform = Platform::new(target_os, target_arch);
        let other_platform = Platform::new(other_os, other_arch);
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
    /// 验证 Platform::release_identifier() 能够为不同的操作系统和架构组合返回正确的发布标识符。
    ///
    /// ## 测试场景
    /// 1. 创建不同 OS 和架构组合的 Platform 实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证标识符格式正确
    ///
    /// ## 预期结果
    /// - macOS x86_64 -> "macOS-Intel"
    /// - macOS aarch64 -> "macOS-AppleSilicon"
    /// - Linux x86_64 -> "Linux-x86_64" 或 "Linux-x86_64-static"
    /// - Linux aarch64 -> "Linux-ARM64"
    /// - Windows x86_64 -> "Windows-x86_64"
    /// - Windows aarch64 -> "Windows-ARM64"
    #[rstest]
    #[case("macos", "x86_64", "macOS-Intel")]
    #[case("macos", "aarch64", "macOS-AppleSilicon")]
    #[case("linux", "x86_64", "Linux-x86_64")]
    #[case("linux", "aarch64", "Linux-ARM64")]
    #[case("windows", "x86_64", "Windows-x86_64")]
    #[case("windows", "aarch64", "Windows-ARM64")]
    fn test_platform_release_identifier_return_ok(
        #[case] os: &str,
        #[case] arch: &str,
        #[case] expected_identifier: &str,
    ) -> Result<()> {
        let platform = Platform::new(os, arch);
        let identifier = platform.release_identifier()?;
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
    /// 验证 Platform::release_identifier() 在不支持的操作系统和架构组合时能够正确返回错误。
    ///
    /// ## 测试场景
    /// 1. 创建不支持的平台组合（unsupported_os + unsupported_arch）
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证返回错误且错误消息包含 "Unsupported platform"
    ///
    /// ## 预期结果
    /// - 返回错误（Result::Err）
    /// - 错误消息包含 "Unsupported platform"
    #[test]
    fn test_platform_release_identifier_with_unsupported_platform_return_ok() -> Result<()> {
        let platform = Platform::new("unsupported_os", "unsupported_arch");
        let result = platform.release_identifier();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported platform"));
        Ok(())
    }

    /// 测试 Platform 实例的 Debug 格式化输出
    ///
    /// ## 测试目的
    /// 验证 Platform 实现了 Debug trait，能够通过 format!("{:?}", platform) 格式化输出调试信息。
    ///
    /// ## 测试场景
    /// 1. 创建 Platform 实例
    /// 2. 使用 Debug 格式化输出
    /// 3. 验证输出包含平台信息（OS 或架构）
    ///
    /// ## 预期结果
    /// - Debug 格式化输出包含平台信息
    /// - 输出中包含操作系统或架构字符串
    #[test]
    fn test_platform_debug_with_platform_instance_return_ok() -> Result<()> {
        let platform = Platform::new("macos", "aarch64");
        let debug_str = format!("{:?}", platform);
        assert!(debug_str.contains("macos") || debug_str.contains("aarch64"));
        Ok(())
    }

    /// 测试克隆 Platform 实例
    ///
    /// ## 测试目的
    /// 验证 Platform 实现了 Clone trait，能够正确克隆 Platform 实例。
    ///
    /// ## 测试场景
    /// 1. 创建 Platform 实例
    /// 2. 调用 clone() 方法克隆实例
    /// 3. 验证克隆实例的属性与原始实例一致
    ///
    /// ## 预期结果
    /// - 成功克隆 Platform 实例
    /// - 克隆实例的 os() 和 arch() 与原始实例相同
    #[test]
    fn test_platform_clone_with_platform_instance_return_ok() -> Result<()> {
        let platform1 = Platform::new("linux", "x86_64");
        let platform2 = platform1.clone();
        assert_eq!(platform1.os(), platform2.os());
        assert_eq!(platform1.arch(), platform2.arch());
        Ok(())
    }

    /// 测试 Platform 实例的相等性比较
    ///
    /// ## 测试目的
    /// 验证 Platform 实现了 PartialEq trait，能够正确比较两个 Platform 实例是否相等。
    ///
    /// ## 测试场景
    /// 1. 创建两个相同 OS 和架构的 Platform 实例
    /// 2. 创建不同 OS 或架构的 Platform 实例
    /// 3. 验证相同实例相等，不同实例不相等
    ///
    /// ## 预期结果
    /// - 相同 OS 和架构的实例相等（== 返回 true）
    /// - 不同 OS 或架构的实例不相等（!= 返回 true）
    #[test]
    fn test_platform_eq_with_same_platforms_return_ok() -> Result<()> {
        let platform1 = Platform::new("macos", "aarch64");
        let platform2 = Platform::new("macos", "aarch64");
        let platform3 = Platform::new("linux", "x86_64");
        assert_eq!(platform1, platform2);
        assert_ne!(platform1, platform3);
        Ok(())
    }

    /// 测试 Linux x86_64 平台检测静态链接需求
    ///
    /// ## 测试目的
    /// 验证在 Linux x86_64 平台上，Platform::release_identifier() 能够检测静态链接需求并返回相应的标识符。
    ///
    /// ## 测试场景
    /// 1. 创建 Linux x86_64 平台实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证标识符为 "Linux-x86_64" 或 "Linux-x86_64-static"
    ///
    /// ## 预期结果
    /// - 返回 "Linux-x86_64" 或 "Linux-x86_64-static"
    /// - 根据是否静态链接返回相应标识符
    #[test]
    fn test_platform_release_identifier_with_linux_x86_64_detects_static_link_return_ok(
    ) -> Result<()> {
        let platform = Platform::new("linux", "x86_64");
        let identifier = platform.release_identifier()?;
        assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
        Ok(())
    }

    /// 测试非 Linux 平台不检查静态链接
    ///
    /// ## 测试目的
    /// 验证在非 Linux 平台（macOS、Windows）上，Platform::release_identifier() 不执行静态链接检查，直接返回标准标识符。
    ///
    /// ## 测试场景
    /// 1. 创建 macOS 和 Windows 平台实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证标识符为标准格式（不包含 static 后缀）
    ///
    /// ## 预期结果
    /// - macOS x86_64 返回 "macOS-Intel"
    /// - Windows x86_64 返回 "Windows-x86_64"
    /// - 不包含 static 后缀
    #[test]
    fn test_platform_release_identifier_with_non_linux_does_not_check_static_return_ok(
    ) -> Result<()> {
        let macos = Platform::new("macos", "x86_64");
        let windows = Platform::new("windows", "x86_64");
        let macos_id = macos.release_identifier()?;
        let windows_id = windows.release_identifier()?;
        assert_eq!(macos_id, "macOS-Intel");
        assert_eq!(windows_id, "Windows-x86_64");
        Ok(())
    }

    /// 测试 Linux 非 x86_64 架构不检查静态链接
    ///
    /// ## 测试目的
    /// 验证在 Linux 非 x86_64 架构（如 ARM64）上，Platform::release_identifier() 不执行静态链接检查。
    ///
    /// ## 测试场景
    /// 1. 创建 Linux ARM64 平台实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证标识符为标准格式（不包含 static 后缀）
    ///
    /// ## 预期结果
    /// - Linux ARM64 返回 "Linux-ARM64"
    /// - 不包含 static 后缀
    #[test]
    fn test_platform_release_identifier_with_linux_non_x86_64_does_not_check_static_return_ok(
    ) -> Result<()> {
        let linux_arm64 = Platform::new("linux", "aarch64");
        let identifier = linux_arm64.release_identifier()?;
        assert_eq!(identifier, "Linux-ARM64");
        Ok(())
    }

    /// 测试所有平台组合返回正确的标识符
    ///
    /// ## 测试目的
    /// 验证 Platform::release_identifier() 对所有支持的操作系统和架构组合都能返回正确的标识符前缀。
    ///
    /// ## 测试场景
    /// 1. 创建所有支持的平台组合（macOS、Linux、Windows × x86_64、aarch64）
    /// 2. 为每个组合调用 release_identifier()
    /// 3. 验证标识符以预期的前缀开头
    ///
    /// ## 预期结果
    /// - macOS x86_64 -> "macOS-Intel"
    /// - macOS aarch64 -> "macOS-AppleSilicon"
    /// - Linux aarch64 -> "Linux-ARM64"
    /// - Windows x86_64 -> "Windows-x86_64"
    /// - Windows aarch64 -> "Windows-ARM64"
    #[test]
    fn test_platform_release_identifier_with_all_combinations_return_collect() -> Result<()> {
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

    /// 测试 Alpine Linux 检测静态链接需求
    ///
    /// ## 测试目的
    /// 验证在 Alpine Linux x86_64 平台上，Platform::release_identifier() 能够检测到静态链接需求。
    ///
    /// ## 测试场景
    /// 1. 创建 Linux x86_64 平台实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证标识符可能包含 static 后缀（如果检测到静态链接）
    ///
    /// ## 预期结果
    /// - 返回 "Linux-x86_64" 或 "Linux-x86_64-static"
    /// - 根据是否检测到静态链接返回相应标识符
    #[test]
    fn test_platform_release_identifier_with_alpine_linux_detects_static_return_ok() -> Result<()> {
        let platform = Platform::new("linux", "x86_64");
        let identifier = platform.release_identifier()?;
        assert!(identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static");
        Ok(())
    }

    /// 测试处理 ldd 命令不同输出的场景
    ///
    /// ## 测试目的
    /// 验证 Platform::release_identifier() 能够正确处理 ldd 命令的不同输出场景，包括不同平台和架构。
    ///
    /// ## 测试场景
    /// 1. 创建 Linux x86_64、macOS x86_64、Linux ARM64 平台实例
    /// 2. 为每个平台调用 release_identifier()
    /// 3. 验证各平台返回正确的标识符
    ///
    /// ## 预期结果
    /// - Linux x86_64 标识符以 "Linux-x86_64" 开头
    /// - macOS x86_64 返回 "macOS-Intel"
    /// - Linux ARM64 返回 "Linux-ARM64"
    #[test]
    fn test_platform_release_identifier_with_ldd_scenarios_handles_different_outputs_return_ok(
    ) -> Result<()> {
        let platform = Platform::new("linux", "x86_64");
        let macos_platform = Platform::new("macos", "x86_64");
        let linux_arm64_platform = Platform::new("linux", "aarch64");
        let identifier = platform.release_identifier()?;
        let macos_id = macos_platform.release_identifier()?;
        let linux_arm64_id = linux_arm64_platform.release_identifier()?;
        assert!(identifier.starts_with("Linux-x86_64"));
        assert_eq!(macos_id, "macOS-Intel");
        assert_eq!(linux_arm64_id, "Linux-ARM64");
        Ok(())
    }

    /// 测试非 Linux 平台不执行静态链接检查
    ///
    /// ## 测试目的
    /// 验证在非 Linux 平台（macOS、Windows）上，Platform::release_identifier() 不执行静态链接检查逻辑。
    ///
    /// ## 测试场景
    /// 1. 创建 macOS 和 Windows 平台实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证标识符为标准格式
    ///
    /// ## 预期结果
    /// - macOS x86_64 返回 "macOS-Intel"
    /// - Windows x86_64 返回 "Windows-x86_64"
    /// - 不执行静态链接检查
    #[test]
    fn test_platform_release_identifier_with_non_linux_return_ok() -> Result<()> {
        let macos = Platform::new("macos", "x86_64");
        let windows = Platform::new("windows", "x86_64");
        let macos_id = macos.release_identifier()?;
        let windows_id = windows.release_identifier()?;
        assert_eq!(macos_id, "macOS-Intel");
        assert_eq!(windows_id, "Windows-x86_64");
        Ok(())
    }

    /// 测试非 x86_64 架构不执行静态链接检查
    ///
    /// ## 测试目的
    /// 验证在非 x86_64 架构（如 ARM64）上，Platform::release_identifier() 不执行静态链接检查。
    ///
    /// ## 测试场景
    /// 1. 创建 Linux ARM64 平台实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证标识符为标准格式
    ///
    /// ## 预期结果
    /// - Linux ARM64 返回 "Linux-ARM64"
    /// - 不执行静态链接检查
    #[test]
    fn test_platform_release_identifier_with_non_x86_64_return_ok() -> Result<()> {
        let linux_arm64 = Platform::new("linux", "aarch64");
        let identifier = linux_arm64.release_identifier()?;
        assert_eq!(identifier, "Linux-ARM64");
        Ok(())
    }

    /// 测试 Linux x86_64 平台处理不同场景（Alpine 检测、静态链接检测）
    ///
    /// ## 测试目的
    /// 验证在 Linux x86_64 平台上，Platform::release_identifier() 能够正确处理不同的检测场景，包括 Alpine Linux 检测和静态链接检测。
    ///
    /// ## 测试场景
    /// 1. 创建 Linux x86_64 平台实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 验证标识符为 "Linux-x86_64" 或 "Linux-x86_64-static"
    ///
    /// ## 预期结果
    /// - 返回 "Linux-x86_64" 或 "Linux-x86_64-static"
    /// - 根据检测结果返回相应标识符
    #[test]
    fn test_platform_release_identifier_with_linux_x86_64_handles_different_scenarios_return_ok(
    ) -> Result<()> {
        let platform = Platform::new("linux", "x86_64");
        let identifier = platform.release_identifier()?;
        assert!(
            identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static",
            "Linux x86_64 platform should return Linux-x86_64 or Linux-x86_64-static, got: {}",
            identifier
        );
        Ok(())
    }

    /// 测试在真实 Linux x86_64 环境中返回有效的标识符
    ///
    /// ## 测试目的
    /// 验证在真实的 Linux x86_64 环境中，Platform::release_identifier() 能够正确检测 Alpine Linux 并返回相应的静态链接标识符。
    ///
    /// ## 测试场景
    /// 1. 创建 Linux x86_64 平台实例
    /// 2. 调用 release_identifier() 获取标识符
    /// 3. 如果检测到 Alpine Linux，验证返回 "Linux-x86_64-static"
    /// 4. 否则验证返回 "Linux-x86_64" 或 "Linux-x86_64-static"
    ///
    /// ## 预期结果
    /// - Alpine Linux 返回 "Linux-x86_64-static"
    /// - 其他 Linux 发行版返回 "Linux-x86_64" 或 "Linux-x86_64-static"
    ///
    /// ## 注意
    /// 此测试仅在 Linux x86_64 系统上执行。
    #[test]
    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "x86_64")]
    fn test_platform_release_identifier_in_actual_linux_environment_return_ok() -> Result<()> {
        let platform = Platform::new("linux", "x86_64");
        let identifier = platform.release_identifier()?;
        assert!(
            identifier == "Linux-x86_64" || identifier == "Linux-x86_64-static",
            "Should return valid Linux x86_64 identifier, got: {}",
            identifier
        );
        if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
            if os_release.contains("Alpine") || os_release.contains("ID=alpine") {
                assert_eq!(
                    identifier, "Linux-x86_64-static",
                    "Alpine Linux should return Linux-x86_64-static"
                );
            }
        }
        Ok(())
    }
}
