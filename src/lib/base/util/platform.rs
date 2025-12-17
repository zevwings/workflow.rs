//! 平台检测工具模块
//!
//! 提供平台检测相关的工具函数，用于识别当前运行的操作系统和架构。

use color_eyre::Result;
use std::env;
use std::fs;
use std::process::Command;

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
    /// use workflow::base::util::Platform;
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
    /// use workflow::base::util::Platform;
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
        if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
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
    /// use workflow::base::util::Platform;
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

/// 检测当前平台并返回 GitHub Releases 格式的平台标识符
///
/// 这是一个便捷函数，等价于 `Platform::detect().release_identifier()`。
///
/// 返回平台标识符字符串，用于匹配 GitHub Releases 中的资源文件。
/// 支持的平台格式：
/// - macOS: `macOS-Intel`, `macOS-AppleSilicon`
/// - Linux: `Linux-x86_64`, `Linux-x86_64-static`, `Linux-ARM64`
/// - Windows: `Windows-x86_64`, `Windows-ARM64`
///
/// 对于 Linux x86_64，会自动检测是否需要静态链接版本（musl/static）：
/// 1. 检查是否是 Alpine Linux（通常使用 musl）
/// 2. 检测当前二进制是否静态链接（使用 `ldd` 命令）
///
/// # 返回
///
/// 返回平台标识符字符串，例如 `"macOS-AppleSilicon"` 或 `"Linux-x86_64-static"`。
///
/// # 错误
///
/// 如果平台不支持，返回相应的错误信息。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::base::util::detect_release_platform;
/// use workflow::log_message;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let platform = detect_release_platform()?;
/// log_message!("Detected platform: {}", platform);
/// # Ok(())
/// # }
/// ```
pub fn detect_release_platform() -> Result<String> {
    Platform::detect().release_identifier()
}
