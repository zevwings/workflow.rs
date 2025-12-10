//! 平台检测工具模块
//!
//! 提供平台检测相关的工具函数，用于识别当前运行的操作系统和架构。

use anyhow::Result;
use std::env;
use std::fs;
use std::process::Command;

/// 检测当前平台并返回 GitHub Releases 格式的平台标识符
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
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let platform = detect_release_platform()?;
/// println!("Detected platform: {}", platform);
/// # Ok(())
/// # }
/// ```
pub fn detect_release_platform() -> Result<String> {
    let arch = env::consts::ARCH;
    let os = env::consts::OS;

    match (os, arch) {
        ("macos", "x86_64") => Ok("macOS-Intel".to_string()),
        ("macos", "aarch64") => Ok("macOS-AppleSilicon".to_string()),
        ("linux", "x86_64") => {
            // 尝试检测是否需要 musl/static 版本
            // 方法1: 检查是否是 Alpine Linux（通常使用 musl）
            if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
                if os_release.contains("Alpine") || os_release.contains("ID=alpine") {
                    return Ok("Linux-x86_64-static".to_string());
                }
            }

            // 方法2: 尝试检测当前二进制是否静态链接
            // 如果 ldd 命令失败或没有输出，可能是静态链接
            if let Ok(output) =
                Command::new("ldd").arg(env::current_exe().unwrap_or_default()).output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // 如果 ldd 输出 "not a dynamic executable" 或 "statically linked"
                // 说明是静态链接，应该使用 static 版本
                if output_str.contains("not a dynamic executable")
                    || output_str.contains("statically linked")
                    || output_str.is_empty()
                {
                    return Ok("Linux-x86_64-static".to_string());
                }
            } else {
                // ldd 命令不存在或失败，可能是 musl 环境（Alpine 等）
                // 在这种情况下，尝试使用 static 版本
                return Ok("Linux-x86_64-static".to_string());
            }

            // 默认返回 glibc 版本（大多数 Linux 发行版）
            Ok("Linux-x86_64".to_string())
        }
        ("linux", "aarch64") => Ok("Linux-ARM64".to_string()),
        ("windows", "x86_64") => Ok("Windows-x86_64".to_string()),
        ("windows", "aarch64") => Ok("Windows-ARM64".to_string()),
        _ => anyhow::bail!("Unsupported platform: {}-{}", os, arch),
    }
}
