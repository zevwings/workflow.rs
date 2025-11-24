//! macOS 系统工具
//!
//! 本模块提供了 macOS 特定的系统操作工具函数。

use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// 移除 macOS 隔离属性（quarantine attribute）
///
/// 在 macOS 上，从网络下载的文件会被 Gatekeeper 添加隔离属性。
/// 这个函数使用 `xattr -d com.apple.quarantine` 移除隔离属性，
/// 允许二进制文件正常执行。
///
/// # 参数
///
/// * `file_path` - 要移除隔离属性的文件路径
///
/// # 返回值
///
/// 如果成功移除隔离属性或文件不存在隔离属性，返回 `Ok(())`。
/// 如果文件不存在，也返回 `Ok(())`（静默处理）。
///
/// # 示例
///
/// ```no_run
/// use std::path::Path;
/// use workflow::base::util::macos::remove_quarantine_attribute;
///
/// let binary_path = Path::new("/usr/local/bin/workflow");
/// remove_quarantine_attribute(binary_path)?;
/// ```
#[cfg(target_os = "macos")]
pub fn remove_quarantine_attribute(file_path: &Path) -> Result<()> {
    // 检查文件是否存在
    if !file_path.exists() {
        return Ok(());
    }

    // 使用 xattr 命令移除隔离属性
    let output = Command::new("xattr")
        .arg("-d")
        .arg("com.apple.quarantine")
        .arg(file_path)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                crate::log_debug!("Removed quarantine attribute from: {}", file_path.display());
            } else {
                // 如果隔离属性不存在，xattr 会返回非零状态码，这是正常的
                let stderr = String::from_utf8_lossy(&result.stderr);
                if !stderr.contains("No such xattr") {
                    crate::log_debug!(
                        "Failed to remove quarantine attribute from {}: {}",
                        file_path.display(),
                        stderr.trim()
                    );
                }
            }
        }
        Err(e) => {
            crate::log_debug!(
                "Failed to execute xattr command for {}: {}",
                file_path.display(),
                e
            );
        }
    }

    Ok(())
}

/// 使用 sudo 移除 macOS 隔离属性（quarantine attribute）
///
/// 这个函数用于移除需要管理员权限的文件（如 `/usr/local/bin` 中的文件）的隔离属性。
/// 使用 `sudo xattr -d com.apple.quarantine` 命令。
///
/// # 参数
///
/// * `file_path` - 要移除隔离属性的文件路径
///
/// # 返回值
///
/// 如果成功移除隔离属性或文件不存在隔离属性，返回 `Ok(())`。
/// 如果文件不存在，也返回 `Ok(())`（静默处理）。
///
/// # 示例
///
/// ```no_run
/// use std::path::Path;
/// use workflow::base::util::macos::remove_quarantine_attribute_with_sudo;
///
/// let binary_path = Path::new("/usr/local/bin/workflow");
/// remove_quarantine_attribute_with_sudo(binary_path)?;
/// ```
#[cfg(target_os = "macos")]
pub fn remove_quarantine_attribute_with_sudo(file_path: &Path) -> Result<()> {
    // 检查文件是否存在
    if !file_path.exists() {
        return Ok(());
    }

    // 使用 sudo xattr 命令移除隔离属性
    let output = Command::new("sudo")
        .arg("xattr")
        .arg("-d")
        .arg("com.apple.quarantine")
        .arg(file_path)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                crate::log_debug!(
                    "Removed quarantine attribute (with sudo) from: {}",
                    file_path.display()
                );
            } else {
                // 如果隔离属性不存在，xattr 会返回非零状态码，这是正常的
                let stderr = String::from_utf8_lossy(&result.stderr);
                if !stderr.contains("No such xattr") {
                    crate::log_debug!(
                        "Failed to remove quarantine attribute (with sudo) from {}: {}",
                        file_path.display(),
                        stderr.trim()
                    );
                }
            }
        }
        Err(e) => {
            crate::log_debug!(
                "Failed to execute sudo xattr command for {}: {}",
                file_path.display(),
                e
            );
        }
    }

    Ok(())
}

